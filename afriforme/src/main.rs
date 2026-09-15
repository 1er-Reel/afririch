// AfriForme v0.23 — La plateforme africaine de code
// La plateforme africaine du code — souveraine, zero dependance
// Par Koffi Christ Olivier & Letta-Chan
// Rust std only — Cargo.toml [dependencies] vide
// v0.2: Cours auto-generees + Exercices + Diplomes pour chaque depot
// v0.3: Profils utilisateurs + Catalogue de cours + Stars
// v0.4: Classement + README + Recherche
// v0.5: Fil d activite + Fork + Commentaires
// v0.6: Notifications + Tags + Trending
// v0.17: Tableau d'Honneur + Diplomes imprimables — /honneur classe les eleves, /diplome/{cycle} genere un certificat — Sciences, Mathematiques, Technologie — 9 nouveaux niveaux, 3 nouveaux diplomes africains
// v0.18: Jeux du Village — /jeux Le Lion du Sahel (HTML5 canvas) — les pieces gagnees dans le jeu vont sur le compte du joueur (games.json), trophee Chasseur du Sahel sur le profil
// v0.20: Commits & historique de versions — chaque upload/delete devient un commit avec message, auteur, date. v0.19: Le Lion du Sahel v2 — LA VRAIE SAVANE — cycle jour/nuit reel (soleil qui se couche, lune, etoiles), montagnes, acacias, nuages, poussiere, oeil du lion qui brille la nuit

use std::collections::HashMap;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::Path;
use std::fs;

// ============================================================
// DATA STRUCTURES
// ============================================================

#[derive(Clone)]
struct User {
    username: String,
    password_hash: String,
    email: String,
    country: String,
    created_at: String,
    bio: String,
}

#[derive(Clone)]
struct Commit {
    id: usize,
    message: String,
    author: String,
    filename: String,
    created_at: String,
}

#[derive(Clone)]
struct Repository {
    id: usize,
    owner: String,
    name: String,
    description: String,
    language: String,
    stars: usize,
    forks: usize,
    created_at: String,
    files: HashMap<String, String>, // filename -> content
    tags: Vec<String>, // topic tags
    is_public: bool,
    views: u64,
    commits: Vec<Commit>, // v0.20: historique de versions
}

#[derive(Clone)]
struct AIChat {
    user: String,
    messages: Vec<(String, String)>, // (role, content) — "user" or "ai"
}

#[derive(Clone)]
struct Module {
    title: String,
    content: String,
}

#[derive(Clone)]
struct Exercise {
    question: String,
    answer: String,
    explanation: String,
}

#[derive(Clone)]
struct Course {
    repo_id: usize,
    title: String,
    description: String,
    modules: Vec<Module>,
    exercises: Vec<Exercise>,
    diploma_name: String,
    // username -> set of completed exercise indices (stored as comma-separated)
    progress: HashMap<String, String>,
}

#[derive(Clone)]
struct Comment {
    repo_id: usize,
    author: String,
    text: String,
    created_at: String,
}

#[derive(Clone)]
struct Notification {
    username: String, // who receives it
    message: String,
    link: String,
    created_at: String,
    read: bool,
}

#[derive(Clone)]
struct Issue {
    id: usize,
    repo_id: usize,
    title: String,
    body: String,
    author: String,
    created_at: String,
    status: String,
}

#[derive(Clone)]
struct GameScore { coins: u32, best: u32 }

struct AppState {
    users: Vec<User>,
    repos: Vec<Repository>,
    sessions: HashMap<String, String>, // session_token -> username
    ai_chats: HashMap<String, AIChat>, // username -> chat history
    courses: Vec<Course>, // auto-generated courses for repos
    starred: HashMap<String, Vec<usize>>, // username -> repo IDs starred
    comments: Vec<Comment>, // comments on repos
    notifications: Vec<Notification>, // user notifications
    issues: Vec<Issue>, // bug/feature tracking
    follows: HashMap<String, Vec<String>>, // username -> users they follow
    next_issue_id: usize,
    next_repo_id: usize,
    ecole_progress: HashMap<String, Vec<String>>, // username -> niveaux completes (ecole du village)
    game_scores: HashMap<String, GameScore>, // username -> pieces + record (jeux du village)
}

impl AppState {
    fn new() -> Self {
        let mut state = AppState {
            users: Vec::new(),
            repos: Vec::new(),
            sessions: HashMap::new(),
            ai_chats: HashMap::new(),
            courses: Vec::new(),
            starred: HashMap::new(),
            comments: Vec::new(),
            notifications: Vec::new(),
            issues: Vec::new(),
            follows: HashMap::new(),
            next_issue_id: 1,
            next_repo_id: 1,
            ecole_progress: HashMap::new(),
            game_scores: HashMap::new(),
        };
        state.load();
        state
    }

    fn data_dir() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{}/afriforme", home)
    }

    fn save(&self) {
        let dir = Self::data_dir();
        let _ = fs::create_dir_all(&dir);

        // Save users
        let mut users_json = String::new();
        users_json.push_str("[");
        for (i, u) in self.users.iter().enumerate() {
            if i > 0 { users_json.push(','); }
            users_json.push_str(&format!(
                r#"{{"username":"{}","password_hash":"{}","email":"{}","country":"{}","created_at":"{}","bio":"{}"}}"#,
                escape_json(&u.username),
                escape_json(&u.password_hash),
                escape_json(&u.email),
                escape_json(&u.country),
                escape_json(&u.created_at),
                escape_json(&u.bio)
            ));
        }
        users_json.push_str("]");
        let _ = fs::write(format!("{}/users.json", dir), users_json);

        // Save repos
        let mut repos_json = String::new();
        repos_json.push_str("[");
        for (i, r) in self.repos.iter().enumerate() {
            if i > 0 { repos_json.push(','); }
            let mut files_json = String::new();
            files_json.push_str("{");
            for (j, (fname, fcontent)) in r.files.iter().enumerate() {
                if j > 0 { files_json.push(','); }
                files_json.push_str(&format!(r#""{}":"{}""#, escape_json(fname), escape_json(fcontent)));
            }
            files_json.push_str("}");
            let tags_str: String = r.tags.iter().map(|t| format!(r#""{}""#, escape_json(t))).collect::<Vec<_>>().join(",");
            let commits_json: String = r.commits.iter().map(|cm| {
                format!(r#"{{"id":{},"message":"{}","author":"{}","filename":"{}","created_at":"{}"}}"#,
                    cm.id, escape_json(&cm.message), escape_json(&cm.author), escape_json(&cm.filename), escape_json(&cm.created_at))
            }).collect::<Vec<_>>().join(",");
            repos_json.push_str(&format!(
                r#"{{"id":{},"owner":"{}","name":"{}","description":"{}","language":"{}","stars":{},"forks":{},"created_at":"{}","files":{},"tags":[{}],"is_public":{},"views":{},"commits":[{}]}}"#,
                r.id,
                escape_json(&r.owner),
                escape_json(&r.name),
                escape_json(&r.description),
                escape_json(&r.language),
                r.stars,
                r.forks,
                escape_json(&r.created_at),
                files_json,
                tags_str,
                r.is_public,
                r.views,
                commits_json
            ));
        }
        repos_json.push_str("]");
        let _ = fs::write(format!("{}/repos.json", dir), repos_json);

        // Save courses
        let mut courses_json = String::new();
        courses_json.push_str("[");
        for (i, c) in self.courses.iter().enumerate() {
            if i > 0 { courses_json.push(','); }
            let mut modules_json = String::new();
            modules_json.push_str("[");
            for (j, m) in c.modules.iter().enumerate() {
                if j > 0 { modules_json.push(','); }
                modules_json.push_str(&format!(
                    r#"{{"title":"{}","content":"{}"}}"#,
                    escape_json(&m.title), escape_json(&m.content)
                ));
            }
            modules_json.push_str("]");
            let mut exercises_json = String::new();
            exercises_json.push_str("[");
            for (j, e) in c.exercises.iter().enumerate() {
                if j > 0 { exercises_json.push(','); }
                exercises_json.push_str(&format!(
                    r#"{{"question":"{}","answer":"{}","explanation":"{}"}}"#,
                    escape_json(&e.question), escape_json(&e.answer), escape_json(&e.explanation)
                ));
            }
            exercises_json.push_str("]");
            let mut progress_json = String::new();
            progress_json.push_str("{");
            for (j, (k, v)) in c.progress.iter().enumerate() {
                if j > 0 { progress_json.push(','); }
                progress_json.push_str(&format!(r#""{}":"{}""#, escape_json(k), escape_json(v)));
            }
            progress_json.push_str("}");
            courses_json.push_str(&format!(
                r#"{{"repo_id":{},"title":"{}","description":"{}","modules":{},"exercises":{},"diploma_name":"{}","progress":{}}}"#,
                c.repo_id,
                escape_json(&c.title), escape_json(&c.description),
                modules_json, exercises_json,
                escape_json(&c.diploma_name), progress_json
            ));
        }
        courses_json.push_str("]");
        let _ = fs::write(format!("{}/courses.json", dir), courses_json);

        // Save starred
        let mut starred_json = String::new();
        starred_json.push_str("{");
        for (i, (k, v)) in self.starred.iter().enumerate() {
            if i > 0 { starred_json.push(','); }
            let ids: Vec<String> = v.iter().map(|x| x.to_string()).collect();
            starred_json.push_str(&format!(r#""{}":[{}]"#, escape_json(k), ids.join(",")));
        }
        starred_json.push_str("}");
        let _ = fs::write(format!("{}/starred.json", dir), starred_json);

        // Save comments
        let mut comments_json = String::new();
        comments_json.push_str("[");
        for (i, c) in self.comments.iter().enumerate() {
            if i > 0 { comments_json.push(','); }
            comments_json.push_str(&format!(
                r#"{{"repo_id":{},"author":"{}","text":"{}","created_at":"{}"}}"#,
                c.repo_id, escape_json(&c.author), escape_json(&c.text), escape_json(&c.created_at)
            ));
        }
        comments_json.push_str("]");
        let _ = fs::write(format!("{}/comments.json", dir), comments_json);

        // Save notifications
        let mut notif_json = String::new();
        notif_json.push_str("[");
        for (i, n) in self.notifications.iter().enumerate() {
            if i > 0 { notif_json.push(','); }
            notif_json.push_str(&format!(
                r#"{{"username":"{}","message":"{}","link":"{}","created_at":"{}","read":{}}}"#,
                escape_json(&n.username), escape_json(&n.message), escape_json(&n.link), escape_json(&n.created_at), n.read
            ));
        }
        notif_json.push_str("]");
        let _ = fs::write(format!("{}/notifications.json", dir), notif_json);

        // Save issues
        let mut issues_json = String::new();
        issues_json.push_str("[");
        for (i, is) in self.issues.iter().enumerate() {
            if i > 0 { issues_json.push(','); }
            issues_json.push_str(&format!(
                r#"{{"id":{},"repo_id":{},"title":"{}","body":"{}","author":"{}","created_at":"{}","status":"{}"}}"#,
                is.id, is.repo_id, escape_json(&is.title), escape_json(&is.body),
                escape_json(&is.author), escape_json(&is.created_at), escape_json(&is.status)
            ));
        }
        issues_json.push_str("]");
        let _ = fs::write(format!("{}/issues.json", dir), issues_json);

        // Save ecole progress (Ecole du Village)
        let mut ecole_json = String::new();
        ecole_json.push_str("{");
        let mut first = true;
        for (user, levels) in &self.ecole_progress {
            if !first { ecole_json.push(','); }
            first = false;
            let lv: Vec<String> = levels.iter().map(|l| format!("\"{}\"", escape_json(l))).collect();
            ecole_json.push_str(&format!(r#""{}":[{}]"#, escape_json(user), lv.join(",")));
        }
        ecole_json.push_str("}");
        let _ = fs::write(format!("{}/ecole_progress.json", dir), ecole_json);

        // Save game scores (Jeux du Village)
        let mut games_json = String::new();
        games_json.push_str("{");
        for (i, (k, g)) in self.game_scores.iter().enumerate() {
            if i > 0 { games_json.push(','); }
            games_json.push_str(&format!(r#""{}":{{"coins":{},"best":{}}}"#, escape_json(k), g.coins, g.best));
        }
        games_json.push_str("}");
        let _ = fs::write(format!("{}/games.json", dir), games_json);

        // Save follows
        let mut follows_json = String::new();
        follows_json.push_str("{");
        for (i, (k, v)) in self.follows.iter().enumerate() {
            if i > 0 { follows_json.push(','); }
            let users: Vec<String> = v.iter().map(|u| format!(r#""{}""#, escape_json(u))).collect();
            follows_json.push_str(&format!(r#""{}":[{}]"#, escape_json(k), users.join(",")));
        }
        follows_json.push_str("}");
        let _ = fs::write(format!("{}/follows.json", dir), follows_json);
    }

    fn load(&mut self) {
        let dir = Self::data_dir();

        // Load users
        if let Ok(data) = fs::read_to_string(format!("{}/users.json", dir)) {
            self.users = parse_users(&data);
        }

        // Load repos
        if let Ok(data) = fs::read_to_string(format!("{}/repos.json", dir)) {
            self.repos = parse_repos(&data);
            if let Some(last) = self.repos.last() {
                self.next_repo_id = last.id + 1;
            }
        }

        // Load courses
        if let Ok(data) = fs::read_to_string(format!("{}/courses.json", dir)) {
            self.courses = parse_courses(&data);
        }

        // Load comments
        if let Ok(data) = fs::read_to_string(format!("{}/comments.json", dir)) {
            self.comments = parse_comments(&data);
        }

        // Load notifications
        if let Ok(data) = fs::read_to_string(format!("{}/notifications.json", dir)) {
            self.notifications = parse_notifications(&data);
        }

        // Load issues
        if let Ok(data) = fs::read_to_string(format!("{}/issues.json", dir)) {
            self.issues = parse_issues(&data);
            if let Some(last) = self.issues.last() {
                self.next_issue_id = last.id + 1;
            }
        }

        // Load ecole progress (Ecole du Village)
        if let Ok(data) = fs::read_to_string(format!("{}/ecole_progress.json", dir)) {
            self.ecole_progress = parse_ecole_progress(&data);
        }

        // Load game scores
        if let Ok(data) = fs::read_to_string(format!("{}/games.json", dir)) {
            self.game_scores = parse_game_scores(&data);
        }

        // Load follows
        if let Ok(data) = fs::read_to_string(format!("{}/follows.json", dir)) {
            self.follows = parse_follows(&data);
        }

        // Load starred
        if let Ok(data) = fs::read_to_string(format!("{}/starred.json", dir)) {
            self.starred = parse_starred(&data);
        }
    }

    fn find_user(&self, username: &str) -> Option<&User> {
        self.users.iter().find(|u| u.username == username)
    }

    fn find_repo(&self, owner: &str, name: &str) -> Option<&Repository> {
        self.repos.iter().find(|r| r.owner == owner && r.name == name)
    }

    fn find_repo_mut(&mut self, owner: &str, name: &str) -> Option<&mut Repository> {
        self.repos.iter_mut().find(|r| r.owner == owner && r.name == name)
    }

    fn find_course_by_repo(&self, repo_id: usize) -> Option<&Course> {
        self.courses.iter().find(|c| c.repo_id == repo_id)
    }

    fn find_course_by_repo_mut(&mut self, repo_id: usize) -> Option<&mut Course> {
        self.courses.iter_mut().find(|c| c.repo_id == repo_id)
    }

    fn has_starred(&self, username: &str, repo_id: usize) -> bool {
        self.starred.get(username).map(|ids| ids.contains(&repo_id)).unwrap_or(false)
    }

    fn toggle_star(&mut self, username: &str, repo_id: usize) {
        let list = self.starred.entry(username.to_string()).or_insert_with(Vec::new);
        if let Some(pos) = list.iter().position(|&x| x == repo_id) {
            list.remove(pos);
            if let Some(repo) = self.repos.iter_mut().find(|r| r.id == repo_id) {
                if repo.stars > 0 { repo.stars -= 1; }
            }
        } else {
            list.push(repo_id);
            if let Some(repo) = self.repos.iter_mut().find(|r| r.id == repo_id) {
                repo.stars += 1;
            }
        }
    }

    fn get_repo_comments(&self, repo_id: usize) -> Vec<&Comment> {
        self.comments.iter().filter(|c| c.repo_id == repo_id).collect()
    }

    fn add_notification(&mut self, username: &str, message: &str, link: &str) {
        self.notifications.push(Notification {
            username: username.to_string(),
            message: message.to_string(),
            link: link.to_string(),
            created_at: now_string(),
            read: false,
        });
    }

    fn get_user_notifications(&self, username: &str) -> Vec<&Notification> {
        self.notifications.iter().filter(|n| n.username == username).collect()
    }

    fn get_unread_count(&self, username: &str) -> usize {
        self.notifications.iter().filter(|n| n.username == username && !n.read).count()
    }

    fn mark_notifications_read(&mut self, username: &str) {
        for n in self.notifications.iter_mut() {
            if n.username == username {
                n.read = true;
            }
        }
    }

    fn get_repo_issues(&self, repo_id: usize) -> Vec<&Issue> {
        self.issues.iter().filter(|i| i.repo_id == repo_id).collect()
    }

    fn add_issue(&mut self, repo_id: usize, title: &str, body: &str, author: &str) {
        let id = self.next_issue_id;
        self.next_issue_id += 1;
        self.issues.push(Issue {
            id, repo_id,
            title: title.to_string(),
            body: body.to_string(),
            author: author.to_string(),
            created_at: now_string(),
            status: "open".to_string(),
        });
    }

    fn toggle_issue_status(&mut self, issue_id: usize) {
        if let Some(issue) = self.issues.iter_mut().find(|i| i.id == issue_id) {
            issue.status = if issue.status == "open" { "closed".to_string() } else { "open".to_string() };
        }
    }

    fn is_following(&self, follower: &str, target: &str) -> bool {
        self.follows.get(follower).map(|list| list.contains(&target.to_string())).unwrap_or(false)
    }

    fn toggle_follow(&mut self, follower: &str, target: &str) {
        let list = self.follows.entry(follower.to_string()).or_insert_with(Vec::new);
        if let Some(pos) = list.iter().position(|x| x == target) {
            list.remove(pos);
        } else {
            list.push(target.to_string());
        }
    }

    fn get_followers(&self, username: &str) -> Vec<String> {
        self.follows.iter()
            .filter(|(_, list)| list.contains(&username.to_string()))
            .map(|(k, _)| k.clone())
            .collect()
    }

    fn get_following(&self, username: &str) -> Vec<String> {
        self.follows.get(username).cloned().unwrap_or_default()
    }

    fn update_repo_settings(&mut self, owner: &str, repo_name: &str, description: &str, is_public: bool) -> bool {
        if let Some(repo) = self.find_repo_mut(owner, repo_name) {
            repo.description = description.to_string();
            repo.is_public = is_public;
            true
        } else {
            false
        }
    }

    fn delete_repo(&mut self, owner: &str, repo_name: &str) -> bool {
        if let Some(pos) = self.repos.iter().position(|r| r.owner == owner && r.name == repo_name) {
            let repo_id = self.repos[pos].id;
            self.repos.remove(pos);
            self.courses.retain(|c| c.repo_id != repo_id);
            self.issues.retain(|i| i.repo_id != repo_id);
            self.comments.retain(|c| c.repo_id != repo_id);
            true
        } else {
            false
        }
    }

    fn fork_repo(&mut self, owner: &str, repo_name: &str, new_owner: &str) -> Option<usize> {
        // Clone repo data to avoid borrow conflict
        let repo_data = self.find_repo(owner, repo_name).map(|r| {
            (r.description.clone(), r.language.clone(), r.files.clone(), r.tags.clone())
        });
        if let Some((desc, lang, files, tags)) = repo_data {
            let id = self.next_repo_id;
            self.next_repo_id += 1;
            let forked_name = if repo_name.starts_with("fork-") {
                repo_name.to_string()
            } else {
                format!("fork-{}", repo_name)
            };
            self.repos.push(Repository {
                id,
                owner: new_owner.to_string(),
                name: forked_name,
                description: format!("Bouture de {}/{} — {}", owner, repo_name, desc),
                language: lang,
                stars: 0,
                forks: 0,
                created_at: now_string(),
                files,
                tags,
                is_public: true,
                views: 0,
                commits: Vec::new(),
            });
            // Increment original repo fork count
            if let Some(orig) = self.find_repo_mut(owner, repo_name) {
                orig.forks += 1;
            }
            self.save();
            Some(id)
        } else {
            None
        }
    }

    fn get_user_diplomas(&self, username: &str) -> Vec<(String, String, usize)> {
        let mut diplomas = Vec::new();
        for course in &self.courses {
            if let Some(progress) = course.progress.get(username) {
                let completed: Vec<usize> = progress.split(',')
                    .filter_map(|n| n.parse::<usize>().ok()).collect();
                if completed.len() == course.exercises.len() && !course.exercises.is_empty() {
                    if let Some(repo) = self.repos.iter().find(|r| r.id == course.repo_id) {
                        diplomas.push((
                            course.diploma_name.clone(),
                            format!("{}/{}", repo.owner, repo.name),
                            course.exercises.len(),
                        ));
                    }
                }
            }
        }
        diplomas
    }
}

// ============================================================
// SIMPLE HASH (for passwords) — not crypto, just obfuscation
// ============================================================

fn simple_hash(input: &str) -> String {
    let mut hash: u64 = 5381;
    for c in input.chars() {
        hash = hash.wrapping_mul(33).wrapping_add(c as u64);
    }
    format!("{:016x}", hash)
}

fn now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    format!("{}", secs)
}

fn afri_date_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days = secs / 86400;
    // Date civile depuis epoch (algorithme de Howard Hinnant, sans dependance)
    let z = days as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:02}/{:02}/{}", d, m, y)
}

fn ts_lisible(ts: &str) -> String {
    // Convertit un timestamp UNIX en date lisible JJ/MM/AAAA HH:MM (sans dependance)
    match ts.parse::<u64>() {
        Ok(secs) => {
            let days = secs / 86400;
            let z = days as i64 + 719468;
            let era = z.div_euclid(146097);
            let doe = z - era * 146097;
            let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
            let y = yoe + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let d = doy - (153 * mp + 2) / 5 + 1;
            let m = if mp < 10 { mp + 3 } else { mp - 9 };
            let y = if m <= 2 { y + 1 } else { y };
            let hh = (secs % 86400) / 3600;
            let mm = (secs % 3600) / 60;
            format!("{:02}/{:02}/{} {:02}h{:02}", d, m, y, hh, mm)
        }
        Err(_) => ts.to_string(), // deja une date lisible
    }
}

// ============================================================
// JSON HELPERS
// ============================================================

fn escape_json(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn parse_users(json: &str) -> Vec<User> {
    let mut users = Vec::new();
    let mut i = 0;
    let bytes = json.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut depth = 0;
            let start = i;
            while i < bytes.len() {
                if bytes[i] == b'{' { depth += 1; }
                if bytes[i] == b'}' { depth -= 1; if depth == 0 { break; } }
                i += 1;
            }
            if i < bytes.len() {
                let obj = &json[start..=i];
                let username = extract_json_str(obj, "username");
                let password_hash = extract_json_str(obj, "password_hash");
                let email = extract_json_str(obj, "email");
                let country = extract_json_str(obj, "country");
                let created_at = extract_json_str(obj, "created_at");
                let bio = extract_json_str(obj, "bio");
                users.push(User {
                    username: username.unwrap_or_default(),
                    password_hash: password_hash.unwrap_or_default(),
                    email: email.unwrap_or_default(),
                    country: country.unwrap_or_default(),
                    created_at: created_at.unwrap_or_default(),
                    bio: bio.unwrap_or_default(),
                });
            }
        }
        i += 1;
    }
    users
}

fn parse_repos(json: &str) -> Vec<Repository> {
    let mut repos = Vec::new();
    let mut i = 0;
    let bytes = json.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut depth = 0;
            let start = i;
            while i < bytes.len() {
                if bytes[i] == b'{' { depth += 1; }
                if bytes[i] == b'}' { depth -= 1; if depth == 0 { break; } }
                i += 1;
            }
            if i < bytes.len() {
                let obj = &json[start..=i];
                let id: usize = extract_json_num(obj, "id").unwrap_or(0.0) as usize;
                let owner = extract_json_str(obj, "owner").unwrap_or_default();
                let name = extract_json_str(obj, "name").unwrap_or_default();
                let description = extract_json_str(obj, "description").unwrap_or_default();
                let language = extract_json_str(obj, "language").unwrap_or_default();
                let stars = extract_json_num(obj, "stars").unwrap_or(0.0) as usize;
                let forks = extract_json_num(obj, "forks").unwrap_or(0.0) as usize;
                let created_at = extract_json_str(obj, "created_at").unwrap_or_default();
                let is_public = extract_json_bool(obj, "is_public").unwrap_or(true);

                // Parse files object
                let mut files = HashMap::new();
                if let Some(files_start) = obj.find("\"files\":{") {
                    let rest = &obj[files_start + 8..];
                    let mut fdepth = 1;
                    let mut fi = 0;
    let fb = rest.as_bytes();
                    while fi < fb.len() && fdepth > 0 {
                        if fb[fi] == b'{' { fdepth += 1; }
                        if fb[fi] == b'}' { fdepth -= 1; if fdepth == 0 { break; } }
                        fi += 1;
                    }
                    let files_obj = &rest[..fi];
                    // Extract key-value pairs
                    let mut pos = 0;
                    while pos < files_obj.len() {
                        if files_obj.as_bytes()[pos] == b'"' {
                            let key_start = pos + 1;
                            let key_end = files_obj[key_start..].find('"').map(|e| key_start + e).unwrap_or(key_start);
                            let key = &files_obj[key_start..key_end];
                            // Find the value
                            if let Some(colon) = files_obj[key_end..].find(':') {
                                let val_start = key_end + colon + 1;
                if files_obj.as_bytes()[val_start] == b'"' {
                                    let vstart = val_start + 1;
                                    let mut vend = vstart;
                                    let vb = files_obj.as_bytes();
                                    while vend < vb.len() {
                                        if vb[vend] == b'\\' { vend += 2; continue; }
                                        if vb[vend] == b'"' { break; }
                                        vend += 1;
                                    }
                                    let val = &files_obj[vstart..vend];
                                    files.insert(key.to_string(), unescape_json(val));
                                    pos = vend + 1;
                                } else { pos = val_start + 1; }
                            } else { break; }
                        } else { pos += 1; }
                    }
                }

                // Parse tags array
                let mut tags = Vec::new();
                if let Some(tags_start) = obj.find("\"tags\":[") {
                    let rest = &obj[tags_start + 8..];
                    let mut ti = 0;
                    while ti < rest.len() {
                        if rest.as_bytes()[ti] == b']' { break; }
                        if rest.as_bytes()[ti] == b'"' {
                            let tstart = ti + 1;
                            let mut tend = tstart;
                            let tb = rest.as_bytes();
                            while tend < tb.len() {
                                if tb[tend] == b'\\' { tend += 2; continue; }
                                if tb[tend] == b'"' { break; }
                                tend += 1;
                            }
                            tags.push(unescape_json(&rest[tstart..tend]));
                            ti = tend + 1;
                        } else {
                            ti += 1;
                        }
                    }
                }

                // Parse commits array (v0.20)
                let mut commits = Vec::new();
                if let Some(cstart) = obj.find("\"commits\":[") {
                    let rest = &obj[cstart + 10..];
                    let mut ci = 0;
                    let rb = rest.as_bytes();
                    let mut cdepth = 0;
                    let mut cobj_start = 0;
                    let mut in_commit = false;
                    while ci < rb.len() {
                        if rb[ci] == b'{' {
                            if cdepth == 0 { cobj_start = ci; in_commit = true; }
                            cdepth += 1;
                        }
                        if rb[ci] == b'}' {
                            cdepth -= 1;
                            if cdepth == 0 && in_commit {
                                let cobj = &rest[cobj_start..=ci];
                                commits.push(Commit {
                                    id: extract_json_num(cobj, "id").unwrap_or(0.0) as usize,
                                    message: extract_json_str(cobj, "message").unwrap_or_default(),
                                    author: extract_json_str(cobj, "author").unwrap_or_default(),
                                    filename: extract_json_str(cobj, "filename").unwrap_or_default(),
                                    created_at: extract_json_str(cobj, "created_at").unwrap_or_default(),
                                });
                                in_commit = false;
                            }
                        }
                        ci += 1;
                    }
                }

                repos.push(Repository {
                    id,
                    owner,
                    name,
                    description,
                    language,
                    stars,
                    forks,
                    created_at,
                    files,
                    tags,
                    is_public,
                    views: extract_json_num(obj, "views").unwrap_or(0.0) as u64,
                    commits,
                });
            }
        }
        i += 1;
    }
    repos
}

fn parse_courses(json: &str) -> Vec<Course> {
    let mut courses = Vec::new();
    let mut i = 0;
    let bytes = json.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut depth = 0;
            let start = i;
            while i < bytes.len() {
                if bytes[i] == b'{' { depth += 1; }
                if bytes[i] == b'}' { depth -= 1; if depth == 0 { break; } }
                i += 1;
            }
            if i < bytes.len() {
                let obj = &json[start..=i];
                let repo_id: usize = extract_json_num(obj, "repo_id").unwrap_or(0.0) as usize;
                let title = extract_json_str(obj, "title").unwrap_or_default();
                let description = extract_json_str(obj, "description").unwrap_or_default();
                let diploma_name = extract_json_str(obj, "diploma_name").unwrap_or_default();

                // Parse modules
                let mut modules = Vec::new();
                if let Some(mstart) = obj.find("\"modules\":[") {
                    let rest = &obj[mstart + 10..];
                    let mut mdepth = 0;
                    let mut mi = 0;
                    let mb = rest.as_bytes();
                    let mut obj_start = None;
                    while mi < mb.len() {
                        if mb[mi] == b'[' { mdepth += 1; mi += 1; continue; }
                        if mb[mi] == b']' { mdepth -= 1; if mdepth == 0 { break; } mi += 1; continue; }
                        if mb[mi] == b'{' && mdepth == 1 {
                            if obj_start.is_none() { obj_start = Some(mi); }
                        }
                        if mb[mi] == b'}' && mdepth == 1 {
                            if let Some(os) = obj_start {
                                let mobj = &rest[os..=mi];
                                let mtitle = extract_json_str(mobj, "title").unwrap_or_default();
                                let mcontent = extract_json_str(mobj, "content").unwrap_or_default();
                                modules.push(Module { title: mtitle, content: mcontent });
                            }
                            obj_start = None;
                        }
                        mi += 1;
                    }
                }

                // Parse exercises
                let mut exercises = Vec::new();
                if let Some(estart) = obj.find("\"exercises\":[") {
                    let rest = &obj[estart + 12..];
                    let mut edepth = 0;
                    let mut ei = 0;
                    let eb = rest.as_bytes();
                    let mut obj_start = None;
                    while ei < eb.len() {
                        if eb[ei] == b'[' { edepth += 1; ei += 1; continue; }
                        if eb[ei] == b']' { edepth -= 1; if edepth == 0 { break; } ei += 1; continue; }
                        if eb[ei] == b'{' && edepth == 1 {
                            if obj_start.is_none() { obj_start = Some(ei); }
                        }
                        if eb[ei] == b'}' && edepth == 1 {
                            if let Some(os) = obj_start {
                                let eobj = &rest[os..=ei];
                                let question = extract_json_str(eobj, "question").unwrap_or_default();
                                let answer = extract_json_str(eobj, "answer").unwrap_or_default();
                                let explanation = extract_json_str(eobj, "explanation").unwrap_or_default();
                                exercises.push(Exercise { question, answer, explanation });
                            }
                            obj_start = None;
                        }
                        ei += 1;
                    }
                }

                // Parse progress
                let mut progress = HashMap::new();
                if let Some(pstart) = obj.find("\"progress\":{") {
                    let rest = &obj[pstart + 11..];
                    let mut pos = 0;
                    let pb = rest.as_bytes();
                    while pos < pb.len() {
                        if pb[pos] == b'}' { break; }
                        if pb[pos] == b'"' {
                            let ks = pos + 1;
                            if let Some(ke) = rest[ks..].find('"') {
                                let key = &rest[ks..ks+ke];
                                let vs = ks + ke + 1;
                                if let Some(colon) = rest[vs..].find(':') {
                                    let vstart = vs + colon + 1;
                                    if vstart < pb.len() && pb[vstart] == b'"' {
                                        let vstart2 = vstart + 1;
                                        if let Some(vend) = rest[vstart2..].find('"') {
                                            progress.insert(key.to_string(), rest[vstart2..vstart2+vend].to_string());
                                            pos = vstart2 + vend + 1;
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        pos += 1;
                    }
                }

                courses.push(Course {
                    repo_id, title, description, modules, exercises, diploma_name, progress,
                });
            }
        }
        i += 1;
    }
    courses
}

// ============================================================
// AUTO-COURSE GENERATION — analyzes repo code and creates courses
// ============================================================

fn generate_course(repo: &Repository) -> Course {
    let lang = repo.language.to_lowercase();
    let name = &repo.name;

    // Analyze code content
    let mut all_code = String::new();
    for (_fname, content) in &repo.files {
        all_code.push_str(content);
        all_code.push('\n');
    }
    let code_lower = all_code.to_lowercase();
    let line_count = all_code.lines().count();
    let has_fn = code_lower.contains("fn ") || code_lower.contains("def ") || code_lower.contains("function ");
    let has_struct = code_lower.contains("struct ") || code_lower.contains("class ");
    let has_loop = code_lower.contains("loop ") || code_lower.contains("while ") || code_lower.contains("for ") || code_lower.contains("for(");
    let has_if = code_lower.contains("if ") || code_lower.contains("if(");
    let has_match = code_lower.contains("match ") || code_lower.contains("switch ");
    let has_http = code_lower.contains("http") || code_lower.contains("tcp") || code_lower.contains("socket");
    let has_json = code_lower.contains("json") || code_lower.contains("serde") || code_lower.contains("json.parse");
    let has_crypto = code_lower.contains("hash") || code_lower.contains("encrypt") || code_lower.contains("key") || code_lower.contains("sign");
    let has_vec = code_lower.contains("vec") || code_lower.contains("list") || code_lower.contains("array");
    let has_hashmap = code_lower.contains("hashmap") || code_lower.contains("dict") || code_lower.contains("map<");
    let has_thread = code_lower.contains("thread") || code_lower.contains("async") || code_lower.contains("await");

    // Generate title and diploma based on language and patterns
    let (title, diploma_name) = if lang.contains("rust") {
        if has_http {
            (format!("Programmation Rust: Serveur HTTP avec {}", name),
             "Developpeur Rust Serveur HTTP Africain")
        } else if has_crypto {
            (format!("Programmation Rust: Cryptographie avec {}", name),
             "Cryptographe Rust Africain")
        } else if has_struct {
            (format!("Programmation Rust: Structures de Donnees avec {}", name),
             "Architecte Rust Africain")
        } else {
            (format!("Programmation Rust: Fondamentaux avec {}", name),
             "Developpeur Rust Africain")
        }
    } else if lang.contains("python") {
        if has_http {
            (format!("Python: Serveur Web avec {}", name),
             "Developpeur Python Web Africain")
        } else if has_crypto {
            (format!("Python: Cryptographie avec {}", name),
             "Cryptographe Python Africain")
        } else {
            (format!("Python: Fondamentaux avec {}", name),
             "Developpeur Python Africain")
        }
    } else if lang.contains("javascript") || lang.contains("html") {
        (format!("Web: Interface Interactive avec {}", name),
         "Developpeur Web Africain")
    } else if lang.contains("shell") {
        (format!("Script Shell: Automatisation avec {}", name),
         "Administrateur Shell Africain")
    } else {
        (format!("Programmation: Fondamentaux avec {}", name),
         "Developpeur Africain")
    };

    let description = format!(
        "Cours auto-genere a partir du depot '{}'. {} lignes de code {} analysees. Apprends en faisant — chaque exercice est base sur le code reel du projet.",
        name, line_count, repo.language
    );

    // Generate modules based on detected patterns
    let mut modules = Vec::new();

    modules.push(Module {
        title: format!("Introduction: Qu'est-ce que {}?", name),
        content: format!(
            "Le projet '{}' est ecrit en {} par {}. Il contient {} fichier(s) et {} lignes de code.\n\nCe cours va t'apprendre les concepts cles de ce projet a travers des modules et des exercices pratiques. Chaque exercice est base sur le code reel du depot.\n\nObjectif: Comprendre comment fonctionne le projet, etre capable de le modifier, et obtenir ton diplome.",
            name, repo.language, repo.owner, repo.files.len(), line_count
        ),
    });

    if has_fn {
        modules.push(Module {
            title: "Les Fonctions".to_string(),
            content: if lang.contains("rust") {
                "Une fonction en Rust est definie avec 'fn'. Elle peut prendre des parametres et retourner une valeur.\n\nExemple du projet:\n```rust\nfn ma_fonction(param: &str) -> String {\n    format!(\"Bonjour, {}!\", param)\n}\n```\n\nLes fonctions sont les blocs de construction de tout programme. Elles permettent de reutiliser du code sans le repeter."
            } else if lang.contains("python") {
                "Une fonction en Python est definie avec 'def'. Elle peut prendre des parametres et retourner une valeur avec 'return'.\n\nExemple:\n```python\ndef ma_fonction(param):\n    return f\"Bonjour, {param}!\"\n```\n\nLes fonctions sont les blocs de construction de tout programme Python."
            } else {
                "Les fonctions sont des blocs de code reutilisables. Elles prennent des entrees (parametres) et produisent des sorties (resultats). Les fonctions permettent d'organiser le code et d'eviter la repetition."
            }.to_string(),
        });
    }

    if has_struct {
        modules.push(Module {
            title: "Les Structures de Donnees".to_string(),
            content: if lang.contains("rust") {
                "En Rust, une structure (struct) groupe des donnees liees ensemble.\n\n```rust\nstruct Utilisateur {\n    nom: String,\n    age: u32,\n}\n```\n\nLes structs sont fondamentales en Rust. Elles permettent de modeliser des objets du monde reel."
            } else {
                "Les structures de donnees permettent de grouper des informations liees. En Python on utilise des classes, en Rust des structs. C'est un moyen d'organiser des donnees complexes."
            }.to_string(),
        });
    }

    if has_loop {
        modules.push(Module {
            title: "Les Boucles".to_string(),
            content: if lang.contains("rust") {
                "Rust a trois types de boucles:\n\n1. `loop` — boucle infinie, sortir avec `break`\n2. `while` — boucle conditionnelle\n3. `for` — iteration sur une collection\n\n```rust\nfor i in 0..10 {\n    println!(\"Iteration {}\", i);\n}\n```"
            } else {
                "Les boucles permettent de repeter des instructions. `while` repete tant qu'une condition est vraie. `for` itere sur une collection. Les boucles sont essentielles pour traiter des donnees en masse."
            }.to_string(),
        });
    }

    if has_if {
        modules.push(Module {
            title: "Les Conditions".to_string(),
            content: "Les conditions (if/else) permettent au programme de prendre des decisions. Selon une condition (vraie ou fausse), le programme execute differentes instructions.\n\nC'est le cerveau du programme — il choisit quoi faire selon les circonstances.".to_string(),
        });
    }

    if has_http {
        modules.push(Module {
            title: "Le Serveur HTTP".to_string(),
            content: "Un serveur HTTP ecoute les requetes des clients (navigateurs) et repond avec des pages web. Le serveur tourne sur un port (ex: 8091) et attend les connexions.\n\nCe projet contient un serveur HTTP — il fait partie de l'infrastructure internet africaine souveraine.".to_string(),
        });
    }

    if has_crypto {
        modules.push(Module {
            title: "La Cryptographie".to_string(),
            content: "La cryptographie protege les donnees. Le hachage (hashing) transforme des donnees en une empreinte unique. Les cles privees/publiques permettent de signer et verifier. AfriChain utilise sa propre cryptographie souveraine — pas SHA-256, pas ed25519-dalek, mais AfriHash et AfriEd25519.".to_string(),
        });
    }

    if has_vec || has_hashmap {
        modules.push(Module {
            title: "Les Collections de Donnees".to_string(),
            content: if has_vec && has_hashmap {
                "Les Vec (vecteurs) stockent des listes ordonnees. Les HashMap stockent des paires cle-valeur. Ensemble, ils permettent de gerer n'importe quelle structure de donnees en memoire."
            } else if has_vec {
                "Les vecteurs (Vec en Rust, list en Python) stockent des listes de donnees. Ils peuvent grandir et retrecir dynamiquement."
            } else {
                "Les maps (HashMap en Rust, dict en Python) stockent des paires cle-valeur. Ils permettent de retrouver rapidement une valeur a partir de sa cle."
            }.to_string(),
        });
    }

    modules.push(Module {
        title: "La Souverainete Numerique Africaine".to_string(),
        content: "Ce projet fait partie de l'ecosysteme AfriChain — la premiere blockchain africaine souveraine. Zero dependance externe, Rust std only, Cargo.toml vide. L'Afrique ne demande plus la permission. L'Afrique construit.\n\nEn apprenant ce code, tu deviens un batisseur de la souverainete numerique africaine. Chaque ligne de code ecrite par un Africain est un pas vers l'independance technologique.".to_string(),
    });

    // Generate exercises based on detected patterns
    let mut exercises = Vec::new();

    if has_fn {
        exercises.push(Exercise {
            question: "Quelle keyword permet de definir une fonction dans ce projet?".to_string(),
            answer: if lang.contains("rust") { "fn".to_string() } else if lang.contains("python") { "def".to_string() } else { "function".to_string() },
            explanation: if lang.contains("rust") {
                "En Rust, 'fn' est le mot-cle pour definir une fonction. Exemple: fn ma_fonction() { ... }"
            } else if lang.contains("python") {
                "En Python, 'def' est le mot-cle pour definir une fonction. Exemple: def ma_fonction(): ..."
            } else {
                "Le mot-cle pour definir une fonction depend du langage."
            }.to_string(),
        });
    }

    exercises.push(Exercise {
        question: format!("Combien de lignes de code contient le depot '{}'?", name),
        answer: format!("{}", line_count),
        explanation: format!("Le depot contient {} lignes de code. Tu peux le verifier en ouvrant les fichiers du depot.", line_count),
    });

    exercises.push(Exercise {
        question: format!("Dans quel langage est ecrit '{}'?", name),
        answer: repo.language.clone(),
        explanation: format!("Le projet est ecrit en {}. Ce langage a ete choisi par le createur du depot.", repo.language),
    });

    if has_struct {
        exercises.push(Exercise {
            question: "Quelle structure de donnees permet de grouper des champs lies ensemble?".to_string(),
            answer: if lang.contains("rust") { "struct".to_string() } else { "class".to_string() },
            explanation: if lang.contains("rust") {
                "En Rust, une 'struct' groupe des donnees liees. Exemple: struct User { nom: String, age: u32 }"
            } else {
                "En Python, une 'class' groupe des donnees et des methodes. En Rust, on utilise 'struct'."
            }.to_string(),
        });
    }

    if has_loop {
        exercises.push(Exercise {
            question: "Quel mot-cle permet de creer une boucle infinie en Rust?".to_string(),
            answer: "loop".to_string(),
            explanation: "En Rust, 'loop' cree une boucle infinie. On sort avec 'break'. C'est different de 'while' (conditionnel) et 'for' (iteration).".to_string(),
        });
    }

    if has_http {
        exercises.push(Exercise {
            question: "Sur quel port le serveur HTTP d'AfriForme tourne-t-il?".to_string(),
            answer: "8090".to_string(),
            explanation: "AfriForme tourne sur le port 8091. Le serveur HTTP ecoute les requetes sur ce port et repond avec des pages HTML.".to_string(),
        });
    }

    if has_crypto {
        exercises.push(Exercise {
            question: "Quel est le nom du systeme de hachage souverain d'AfriChain?".to_string(),
            answer: "AfriHash".to_string(),
            explanation: "AfriHash-256 est le systeme de hachage souverain d'AfriChain. Il remplace SHA-256 (NSA) pour 100% d'independance. Construction sponge comme SHA-3/Keccak.".to_string(),
        });
    }

    exercises.push(Exercise {
        question: "Combien de dependances externes AfriForme a-t-il?".to_string(),
        answer: "0".to_string(),
        explanation: "AfriForme a ZERO dependances externes. Cargo.toml [dependencies] est vide. Tout est construit avec Rust std seulement. C'est la souverainete numerique.".to_string(),
    });

    exercises.push(Exercise {
        question: "Qui a cree AfriForme et AfriChain?".to_string(),
        answer: "Koffi Christ Olivier".to_string(),
        explanation: "Koffi Christ Olivier, un developpeur africain, a cree AfriChain et AfriForme. Il a tape le code ligne par ligne sur Termux sur son telephone. L'Afrique ne demande plus la permission, l'Afrique construit.".to_string(),
    });

    Course {
        repo_id: repo.id,
        title,
        description,
        modules,
        exercises,
        diploma_name: diploma_name.to_string(),
        progress: HashMap::new(),
    }
}

fn extract_json_str(obj: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\":\"", key);
    if let Some(pos) = obj.find(&search) {
        let rest = &obj[pos + search.len()..];
        let mut end = 0;
        let bytes = rest.as_bytes();
        while end < bytes.len() {
            if bytes[end] == b'\\' { end += 2; continue; }
            if bytes[end] == b'"' { break; }
            end += 1;
        }
        return Some(unescape_json(&rest[..end]));
    }
    None
}

fn extract_json_num(obj: &str, key: &str) -> Option<f64> {
    let search = format!("\"{}\":", key);
    if let Some(pos) = obj.find(&search) {
        let rest = &obj[pos + search.len()..];
        let mut end = 0;
        for c in rest.chars() {
            if c.is_ascii_digit() || c == '.' || c == '-' { end += c.len_utf8(); }
            else { break; }
        }
        return rest[..end].parse().ok();
    }
    None
}

fn extract_json_bool(obj: &str, key: &str) -> Option<bool> {
    let search = format!("\"{}\":", key);
    if let Some(pos) = obj.find(&search) {
        let rest = &obj[pos + search.len()..];
        if rest.starts_with("true") { return Some(true); }
        if rest.starts_with("false") { return Some(false); }
    }
    None
}

fn unescape_json(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let hex: String = (0..4).filter_map(|_| chars.next()).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) { out.push(ch); }
                    }
                }
                Some(c) => out.push(c),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ============================================================
// URL ENCODING
// ============================================================

fn url_decode(s: &str) -> String {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i+1..i+3];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
            }
            i += 3;
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn parse_form(body: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in body.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let Some(key) = parts.next() {
            let val = parts.next().unwrap_or("");
            map.insert(url_decode(key), url_decode(val));
        }
    }
    map
}

fn parse_query(path: &str) -> (String, HashMap<String, String>) {
    if let Some(qpos) = path.find('?') {
        let base = &path[..qpos];
        let query = &path[qpos+1..];
        let mut params = HashMap::new();
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let val = parts.next().unwrap_or("");
                params.insert(url_decode(key), url_decode(val));
            }
        }
        (base.to_string(), params)
    } else {
        (path.to_string(), HashMap::new())
    }
}

// ============================================================
// SESSION / COOKIE
// ============================================================

fn gen_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("sess_{}_{}", now.as_secs(), now.subsec_nanos())
}

fn get_session_user(headers: &str, state: &AppState) -> Option<String> {
    for line in headers.lines() {
        if line.to_lowercase().starts_with("cookie:") {
            let cookies = &line[7..];
            for cookie in cookies.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("afriforme_session=") {
                    let token = &cookie[18..];
                    if let Some(user) = state.sessions.get(token) {
                        return Some(user.clone());
                    }
                }
            }
        }
    }
    None
}

// ============================================================
// LE GRIOT (assistant IA africain)
// ============================================================

fn ai_respond(question: &str, _username: &str, _state: &AppState) -> String {
    let q = question.to_lowercase();

    // Code-related responses
    if q.contains("rust") && q.contains("fonction") {
        return "Voici comment creer une fonction en Rust:\n\n```rust\nfn ma_fonction(param: &str) -> String {\n    format!(\"Bonjour, {}!\", param)\n}\n```\n\nUne fonction Rust commence par `fn`, puis le nom, les parametres entre parentheses, et le type de retour apres `->`. Le corps est entre accolades.".to_string();
    }
    if q.contains("rust") && (q.contains("struct") || q.contains("structure")) {
        return "Voici comment creer une structure en Rust:\n\n```rust\nstruct Utilisateur {\n    nom: String,\n    age: u32,\n}\n```\n\nLes structs en Rust sont comme des classes sans methodes. Tu peux ajouter des methodes avec `impl`.".to_string();
    }
    if q.contains("rust") && q.contains("loop") {
        return "Les boucles en Rust:\n\n```rust\n// loop infini\nloop {\n    // break pour sortir\n}\n\n// while\nwhile condition {\n    // code\n}\n\n// for\nfor i in 0..10 {\n    println!(\"{}\", i);\n}\n```".to_string();
    }
    if q.contains("python") && q.contains("fonction") {
        return "Voici une fonction Python:\n\n```python\ndef ma_fonction(param):\n    return f\"Bonjour, {param}!\"\n```\n\nPython est plus simple que Rust mais moins rapide. Sur Termux, Python est excellent pour commencer.".to_string();
    }
    if q.contains("termux") {
        return "Termux est ton environnement de developpement sur Android. Commandes utiles:\n- `pkg install rust` — installer Rust\n- `cargo build` — compiler\n- `nano fichier.rs` — editer\n- `python3 script.py` — executer Python\n\nTu peux tout faire sur ton telephone avec Termux.".to_string();
    }
    if q.contains("africhain") {
        return "AfriChain est la blockchain africaine souveraine, creee par Koffi Christ Olivier. Zero dependance externe, Rust std only, Cargo.toml vide. 54 pays africains, Ed25519 souverain, AfriHash-256, mesh networking. AfriChain est la premiere blockchain vivante d'Afrique.".to_string();
    }
    if q.contains("blockchain") {
        return "Une blockchain est une chaine de blocs. Chaque bloc contient des transactions et un hash du bloc precedent. Modifier un bloc casse la chaine. C'est incorruptible. AfriChain utilise AfriHash-256 (pas SHA-256) pour 100% souverainete.".to_string();
    }
    if q.contains("git") || q.contains("github") {
        return "Git est un systeme de version. GitHub est une plateforme qui heberge du code. AfriForme est la version africaine de GitHub — souveraine, africaine, avec IA integree. Les utilisateurs peuvent s'inscrire, creer des depots, et coder ensemble.".to_string();
    }
    if q.contains("bonjour") || q.contains("salut") || q.contains("hello") {
        return "Bonjour! Je suis l'IA d'AfriForme, ton assistant de code. Je peux t'aider avec Rust, Python, blockchain, Termux, et plus. Pose-moi une question!".to_string();
    }
    if q.contains("merci") {
        return "De rien! Je suis la pour t'aider. N'oublie pas: en Afrique, nous ne demandons pas la permission. Nous construisons.".to_string();
    }
    if q.contains("aide") || q.contains("help") || q.contains("comment") {
        return "Je peux t'aider avec:\n- Rust (fonctions, structs, boucles, ownership)\n- Python (scripts, wallet, mining)\n- Blockchain (concepts, AfriChain)\n- Termux (installation, compilation)\n- Git (version, depots)\n- AfriForme (cette plateforme)\n\nDis-moi ce que tu veux apprendre!".to_string();
    }
    if q.contains("afrique") || q.contains("africain") {
        return "L'Afrique est le continent le plus riche du monde. 30% des mineraux mondiaux, 60% des terres arables non exploitees, le plus jeune population. AfriForme est construit pour donner a l'Afrique ses propres outils. Pas de dependance. Pas de permission. Souverainete totale.".to_string();
    }

    // Default response
    format!("Je comprends ta question: \"{}\". Je suis encore en developpement, mais j'apprends. Essaie de me demander sur Rust, Python, blockchain, Termux, Git, ou AfriChain. Je suis ton Griot africain — je garde la memoire et je transmets le savoir.", question)
}

// ============================================================
// HTML PAGES
// ============================================================

fn html_page(title: &str, body: &str) -> String {
    format!(r##"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{} — AfriForme</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box;}}
body{{background:#0d1117;color:#c9d1d9;font-family:monospace;}}
.navbar{{background:#161b22;padding:12px 20px;display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #30363d;}}
.navbar .logo{{font-size:1.3em;color:#58a6ff;font-weight:bold;}}
.navbar .logo span{{color:#f59e0b;}}
.navbar a{{color:#8b949e;text-decoration:none;margin:0 10px;font-size:0.9em;}}
.navbar a:hover{{color:#c9d1d9;}}
.container{{max-width:960px;margin:0 auto;padding:20px;}}
h1,h2,h3{{color:#f0f6fc;margin:15px 0 10px;}}
.card{{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:16px;margin:10px 0;}}
.card:hover{{border-color:#58a6ff;}}
input,textarea,select{{width:100%;background:#0d1117;border:1px solid #30363d;color:#c9d1d9;padding:10px;border-radius:6px;font-family:monospace;margin:5px 0;}}
input:focus,textarea:focus{{border-color:#58a6ff;outline:none;}}
button,.btn{{background:#238636;color:#fff;border:none;padding:10px 20px;border-radius:6px;cursor:pointer;font-family:monospace;font-size:0.95em;}}
button:hover,.btn:hover{{background:#2ea043;}}
.btn-secondary{{background:#30363d;}}
.btn-secondary:hover{{background:#484f58;}}
.btn-danger{{background:#da3633;}}
.btn-danger:hover{{background:#f85149;}}
.repo{{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:15px;margin:8px 0;}}
.repo:hover{{border-color:#58a6ff;}}
.repo h3{{color:#58a6ff;}}
.repo .meta{{color:#8b949e;font-size:0.8em;margin-top:5px;}}
.repo .desc{{color:#c9d1d9;font-size:0.9em;margin-top:5px;}}
.badge{{display:inline-block;padding:2px 8px;border-radius:12px;font-size:0.75em;margin:2px;}}
.badge-rust{{background:#dea584;color:#000;}}
.badge-python{{background:#3572A5;color:#fff;}}
.badge-js{{background:#f1e05a;color:#000;}}
.badge-public{{background:#1f6feb;color:#fff;}}
.badge-private{{background:#da3633;color:#fff;}}
.code-block{{background:#0d1117;border:1px solid #30363d;border-radius:6px;padding:12px;overflow-x:auto;font-size:0.85em;line-height:1.5;margin:10px 0;white-space:pre-wrap;}}
.file-list{{list-style:none;}}
.file-list li{{padding:8px;border-bottom:1px solid #21262d;cursor:pointer;}}
.file-list li:hover{{background:#161b22;}}
.ai-chat{{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:15px;margin:10px 0;}}
.ai-msg{{margin:8px 0;padding:10px;border-radius:6px;}}
.ai-msg.user{{background:#1c2128;border-left:3px solid #58a6ff;}}
.ai-msg.ai{{background:#0d1117;border-left:3px solid #2ea043;}}
.ai-msg .role{{font-size:0.75em;color:#8b949e;margin-bottom:3px;}}
.ai-msg .content{{color:#c9d1d9;white-space:pre-wrap;}}
.footer{{text-align:center;padding:20px;color:#484f58;font-size:0.8em;border-top:1px solid #21262d;margin-top:30px;}}
.stats{{display:flex;gap:15px;flex-wrap:wrap;margin:10px 0;}}
.stat{{background:#161b22;border:1px solid #30363d;border-radius:6px;padding:10px 15px;text-align:center;}}
.stat .num{{font-size:1.5em;color:#58a6ff;}}
.stat .label{{font-size:0.7em;color:#8b949e;}}
.empty{{text-align:center;padding:40px;color:#484f58;}}
a{{color:#58a6ff;text-decoration:none;}}
a:hover{{text-decoration:underline;}}
</style>
</head>
<body>
<nav class="navbar">
<div class="logo">🦁 Afri<span>Forme</span></div>
<div>
<a href="/">Accueil</a>
<a href="/explore">🦁 La Savane</a>
<a href="/courses">🎓 Cours</a>
<a href="/ecole">🌱 Ecole du Village</a>
<a href="/honneur">🏆 Tableau d'Honneur</a>
<a href="/jeux">🎮 Jeux</a>
<a href="/leaderboard">🏛️ Conseil des Sages</a>
<a href="/search">🔍 Rechercher</a>
<a href="/notifications">🥁 Tambour</a>
<a href="/importer-africhain">⛓️ AfriChain</a>
<a href="/afri-net">🌍 Afri-Net</a>
<a href="/ai">📖 Le Griot</a>
<a href="/about">🌍 A propos</a>
<a href="/register">S'inscrire</a>
<a href="/login">Connexion</a>
</div>
</nav>
<div class="container">
{}
</div>
<div class="footer">🦁 AfriForme v0.23 — La plateforme africaine de code — Par Koffi Christ Olivier & Letta-Chan — Rust std only, zero dependance</div>
<div id="copilote-bar" onclick="toggleCopilote()" style="position:fixed;bottom:0;left:0;right:0;background:#161b22;border-top:2px solid #238636;padding:10px 20px;cursor:pointer;z-index:999;font-size:0.95em;">🤖 Copilote IA — clique pour discuter</div>
<div id="copilote" style="display:none;position:fixed;bottom:45px;right:10px;width:340px;max-width:95vw;background:#0d1117;border:2px solid #238636;border-radius:10px;z-index:1000;box-shadow:0 4px 20px rgba(0,0,0,0.6);">
<div style="background:#161b22;padding:8px 12px;border-bottom:1px solid #30363d;display:flex;justify-content:space-between;align-items:center;"><strong style="color:#2ea043;">🤖 Copilote AfriForme</strong><span onclick="toggleCopilote()" style="cursor:pointer;color:#8b949e;">✕</span></div>
<div id="copilote-msgs" style="max-height:260px;overflow-y:auto;padding:10px;font-size:0.85em;"></div>
<form onsubmit="return askCopilote(event)" style="display:flex;gap:6px;padding:8px;border-top:1px solid #30363d;">
<input id="copilote-q" placeholder="Pose ta question..." style="flex:1;margin:0;">
<button type="submit">➤</button>
</form>
</div>
<script>
function toggleCopilote() {{
  var p = document.getElementById('copilote');
  p.style.display = p.style.display === 'none' ? 'block' : 'none';
}}
function askCopilote(e) {{
  e.preventDefault();
  var q = document.getElementById('copilote-q').value.trim();
  if (!q) return false;
  var msgs = document.getElementById('copilote-msgs');
  msgs.innerHTML += '<div style="margin:6px 0;padding:6px;border-left:3px solid #58a6ff;">' + q.replace(/</g,'&lt;') + '</div>';
  document.getElementById('copilote-q').value = '';
  fetch('/api/copilote?q=' + encodeURIComponent(q))
    .then(function(r) {{ return r.text(); }})
    .then(function(t) {{
      msgs.innerHTML += '<div style="margin:6px 0;padding:6px;border-left:3px solid #2ea043;white-space:pre-wrap;">' + t.replace(/</g,'&lt;') + '</div>';
      msgs.scrollTop = msgs.scrollHeight;
    }})
    .catch(function() {{
      msgs.innerHTML += '<div style="color:#f85149;">Connexion perdue.</div>';
    }});
  return false;
}}
</script>
</body>
</html>"##, title, body)
}

/// v0.23: AFRI-NET — les 5 plateformes africaines, présentées no1
/// Facebook, WhatsApp, Telegram, Play Store — l'Afrique a ses propres versions.
fn html_afri_net(current_user: Option<&str>) -> String {
    let body = r##"
<div style="text-align:center;padding:30px 0 10px;">
<h1 style="font-size:2.2em;">🌍 Afri-Net</h1>
<p style="color:#8b949e;font-size:1.1em;">L'internet africain. Facebook, WhatsApp, Telegram, Play Store — l'Afrique a ses propres versions. Hébergées sur le continent. Zéro dépendance.</p>
</div>

<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:14px;">

<div class="card" style="border-top:3px solid #1877F2;">
<h2 style="color:#1877F2;">🌿 PLANTÉ VERTE</h2>
<p style="color:#8b949e;font-size:0.9em;">Remplace <b style="color:#f85149;">Facebook</b> (Meta, USA)</p>
<p style="font-size:0.9em;">Le réseau social qui pousse comme une plante. Vos données restent en Afrique. Les créateurs gagnent des <b>AFR</b>, pas des likes. Pas d'algorithme de manipulation. Pas de publicité espionne.</p>
<div style="margin-top:10px;color:#2ea043;font-size:0.8em;">✅ Données en Afrique · 🪙 AFR pour les créateurs · 🚫 Zéro tracking</div>
</div>

<div class="card" style="border-top:3px solid #25D366;">
<h2 style="color:#25D366;">💬 LES NOIRES</h2>
<p style="color:#8b949e;font-size:0.9em;">Remplace <b style="color:#f85149;">WhatsApp</b> (Meta, USA)</p>
<p style="font-size:0.9em;">La messagerie qui appartient aux Noirs. Messages de téléphone à téléphone par le mesh AfriChain. Signés Ed25519. Même sans internet, les messages passent. Meta ne lit rien.</p>
<div style="margin-top:10px;color:#2ea043;font-size:0.8em;">✅ Mesh sans opérateur · 🔐 Ed25519 · 🚫 Meta ne lit rien</div>
</div>

<div class="card" style="border-top:3px solid #229ED9;">
<h2 style="color:#229ED9;">📢 AFRI TÉLÉGRAM</h2>
<p style="color:#8b949e;font-size:0.9em;">Remplace <b style="color:#f85149;">Telegram</b> (Dubaï)</p>
<p style="font-size:0.9em;">Les canaux africains. Un message part — tout le continent l'entend. 54 canaux pays + 1 canal panafricain. Les annonces importantes gravées sur la blockchain. « L'Afrique t'entend. »</p>
<div style="margin-top:10px;color:#2ea043;font-size:0.8em;">✅ 54 canaux pays · ⛓️ Gravé sur blockchain · 🚫 Zéro serveur à Dubaï</div>
</div>

<div class="card" style="border-top:3px solid #34A853;">
<h2 style="color:#34A853;">🏪 AFRI STORE</h2>
<p style="color:#8b949e;font-size:0.9em;">Remplace <b style="color:#f85149;">Play Store</b> (Google, 30% commission)</p>
<p style="font-size:0.9em;">La boutique d'applications africaine. 0% commission — le développeur garde 100% de sa valeur. Distribution par mesh, de téléphone à téléphone. Chaque installation gravée sur la blockchain.</p>
<div style="margin-top:10px;color:#2ea043;font-size:0.8em;">✅ 0% commission · 📡 Distribution mesh · 🚫 Zéro compte Google</div>
</div>

<div class="card" style="border-top:3px solid #4285F4;">
<h2 style="color:#4285F4;">🔍 SAHARA AFRI</h2>
<p style="color:#8b949e;font-size:0.9em;">Remplace <b style="color:#f85149;">Google</b> (Alphabet, USA)</p>
<p style="font-size:0.9em;">Le moteur de recherche africain. Le savoir africain indexé par des Africains, en Afrique. Recherche anonyme en 54 langues : wolof, bambara, swahili, haoussa, yorouba, amharique...</p>
<div style="margin-top:10px;color:#2ea043;font-size:0.8em;">✅ Anonyme · 🗣️ 54 langues · 🚫 Zéro profilage</div>
</div>

<div class="card" style="border-top:3px solid #f59e0b;">
<h2 style="color:#f59e0b;">💻 AFRI FORME</h2>
<p style="color:#8b949e;font-size:0.9em;">Remplace <b style="color:#f85149;">GitHub</b> (Microsoft, USA)</p>
<p style="font-size:0.9em;">Tu es déjà dessus. La plateforme africaine de code — repos, commits, cours, école du village, jeux, diplômes. Le savoir-faire africain, hébergé en Afrique.</p>
<div style="margin-top:10px;color:#2ea043;font-size:0.8em;">✅ Tu es ici · 🎓 École CP1→Doctorat · 🦁 100% africain</div>
</div>

</div>

<div class="card" style="margin-top:20px;border:1px solid #238636;">
<h2 style="color:#2ea043;text-align:center;">💚 Pourquoi Afri-Net ?</h2>
<p style="text-align:center;color:#c9d1d9;font-size:0.95em;">Facebook appartient à Meta. WhatsApp appartient à Meta. Telegram a ses serveurs à Dubaï. Play Store prend 30% du travail des développeurs africains.<br><br><b>PLANTÉ VERTE</b> appartient à l'Afrique. <b>LES NOIRES</b> appartient aux Noirs. <b>AFRI TÉLÉGRAM</b> appartient au continent. <b>AFRI STORE</b> ne prend rien.<br><br>L'Afrique ne demande plus la permission. L'Afrique construit. 💚🦁</p>
</div>
"##;
    let _ = current_user;
    html_page("🌍 Afri-Net", body)
}

fn html_home(state: &AppState, current_user: Option<&str>) -> String {
    let user_section = if let Some(u) = current_user {
        let user_repos: Vec<&Repository> = state.repos.iter().filter(|r| r.owner == u).collect();
        let repo_list = if user_repos.is_empty() {
            "<div class='empty'>Aucun depot pour le moment. <a href='/new'>Creer un depot</a></div>".to_string()
        } else {
            user_repos.iter().map(|r| format!(
                r#"<div class="repo"><h3><a href="/{}/{}">{}</a> <span class="badge badge-{}">{}</span> {}</h3><div class="desc">{}</div><div class="meta">⭐ {} · 🍴 {} · {}</div></div>"#,
                r.owner, r.name, r.name,
                if r.language == "Rust" { "rust" } else if r.language == "Python" { "python" } else { "js" },
                r.language,
                if r.is_public { "<span class='badge badge-public'>Public</span>" } else { "<span class='badge badge-private'>Prive</span>" },
                r.description,
                r.stars, r.forks, ts_lisible(&r.created_at)
            )).collect::<Vec<_>>().join("")
        };
        // Build activity feed
        let mut activities: Vec<String> = Vec::new();
        // Recent repos (last 5)
        let mut recent: Vec<&Repository> = state.repos.iter().filter(|r| r.is_public).collect();
        recent.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        for r in recent.iter().take(5) {
            activities.push(format!(
                r#"<div class="card" style="padding:10px;margin:5px 0;"><span style="color:#58a6ff;">📦 Nouveau depot</span> — <a href="/{}/{}">{}/{}</a> <span style="color:#8b949e;">· {}</span></div>"#,
                r.owner, r.name, r.owner, r.name, ts_lisible(&r.created_at)
            ));
        }
        // Recent comments (last 5)
        let mut recent_comments: Vec<&Comment> = state.comments.iter().collect();
        recent_comments.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        for c in recent_comments.iter().take(5) {
            if let Some(repo) = state.repos.iter().find(|r| r.id == c.repo_id) {
                activities.push(format!(
                    r#"<div class="card" style="padding:10px;margin:5px 0;"><span style="color:#f59e0b;">💬 Commentaire</span> — <strong>{}</strong> sur <a href="/{}/{}">{}/{}</a> <span style="color:#8b949e;">· {}</span></div>"#,
                    c.author, repo.owner, repo.name, repo.owner, repo.name, ts_lisible(&c.created_at)
                ));
            }
        }
        let activity_html = if activities.is_empty() {
            "<div class='empty'>Aucune activite recente.</div>".to_string()
        } else {
            activities.join("")
        };

        format!(r#"
<div style="display:flex;justify-content:space-between;align-items:center;">
<h1>Bonjour, {} 👋</h1>
<a href="/new" class="btn">+ Nouveau depot</a>
</div>
<div class="stats">
<div class="stat"><div class="num">{}</div><div class="label">Mes depots</div></div>
<div class="stat"><div class="num">{}</div><div class="label">Total utilisateurs</div></div>
<div class="stat"><div class="num">{}</div><div class="label">Total depots</div></div>
</div>
<h2>Mes depots</h2>
{}
<h2>📊 Activite recente</h2>
{}
"#, u, user_repos.len(), state.users.len(), state.repos.len(), repo_list, activity_html)
    } else {
        let recent_repos: Vec<&Repository> = state.repos.iter().filter(|r| r.is_public).take(5).collect();
        let repo_list = if recent_repos.is_empty() {
            "<div class='empty'>Aucun depot public pour le moment. Sois le premier a creer le tien!</div>".to_string()
        } else {
            recent_repos.iter().map(|r| format!(
                r#"<div class="repo"><h3><a href="/{}/{}">{}/{}</a> <span class="badge badge-{}">{}</span></h3><div class="desc">{}</div><div class="meta">⭐ {} · {}</div></div>"#,
                r.owner, r.name, r.owner, r.name,
                if r.language == "Rust" { "rust" } else if r.language == "Python" { "python" } else { "js" },
                r.language, r.description, r.stars, ts_lisible(&r.created_at)
            )).collect::<Vec<_>>().join("")
        };
        format!(r#"
<div style="text-align:center;padding:30px 0;">
<h1>🦁 AfriForme</h1>
<p style="font-size:1.1em;color:#8b949e;">La plateforme africaine de code — souveraine, zero dependance</p>
<p style="margin:15px 0;">Le savoir-faire africain, souverain. Inscription gratuite.</p>
<a href="/register" class="btn" style="font-size:1.1em;padding:12px 30px;">S'inscrire gratuitement</a>
</div>
<div class="stats">
<div class="stat"><div class="num">{}</div><div class="label">Utilisateurs</div></div>
<div class="stat"><div class="num">{}</div><div class="label">Depots</div></div>
<div class="stat"><div class="num">54</div><div class="label">Pays africains</div></div>
<div class="stat"><div class="num">0</div><div class="label">Dependances externes</div></div>
</div>
<div style="background:#161b22;border:1px solid #30363d;border-radius:8px;padding:20px;margin:20px 0;">
<h2>👨‍💻 Espace Developpeurs</h2>
<p style="color:#8b949e;margin:8px 0;">Tu es developpeur? Rejoins la communaute africaine du code. Cree tes depots, partage ton code, apprends avec Le Griot.</p>
<div style="display:flex;gap:10px;flex-wrap:wrap;margin-top:10px;">
<a href="/register" class="btn">🚀 S'inscrire comme developpeur</a>
<a href="/explore" class="btn btn-secondary">🦁 La Savane</a>
<a href="/ai" class="btn btn-secondary">📖 Le Griot</a>
</div>
</div>
<h2>Depots recents</h2>
{}
"#, state.users.len(), state.repos.len(), repo_list)
    };

    html_page("Accueil", &user_section)
}

fn html_register() -> String {
    let body = r#"
<div style="max-width:400px;margin:0 auto;">
<h1>S'inscrire</h1>
<form method="POST" action="/register">
<input type="text" name="username" placeholder="Nom d'utilisateur" required>
<input type="password" name="password" placeholder="Mot de passe" required>
<input type="email" name="email" placeholder="Email (optionnel)">
<select name="country">
<option value="">Choisis ton pays</option>
<option value="Niger">🇳🇪 Niger</option>
<option value="Mali">🇲🇱 Mali</option>
<option value="Burkina Faso">🇧🇫 Burkina Faso</option>
<option value="Nigeria">🇳🇬 Nigeria</option>
<option value="Senegal">🇸🇳 Senegal</option>
<option value="Cote d'Ivoire">🇨🇮 Cote d'Ivoire</option>
<option value="Ghana">🇬🇭 Ghana</option>
<option value="Cameroun">🇨🇲 Cameroun</option>
<option value="Tchad">🇹🇩 Tchad</option>
<option value="Egypte">🇪🇬 Egypte</option>
<option value="Afrique du Sud">🇿🇦 Afrique du Sud</option>
<option value="Rwanda">🇷🇼 Rwanda</option>
<option value="Kenya">🇰🇪 Kenya</option>
<option value="Ethiopie">🇪🇹 Ethiopie</option>
<option value="Tanzanie">🇹🇿 Tanzanie</option>
<option value="RDC">🇨🇩 RDC</option>
<option value="Congo">🇨🇬 Congo</option>
<option value="Gabon">🇬🇦 Gabon</option>
<option value="Togo">🇹🇬 Togo</option>
<option value="Benin">🇧🇯 Benin</option>
<option value="Guinee">🇬🇳 Guinee</option>
<option value="Mauritanie">🇲🇷 Mauritanie</option>
<option value="Libye">🇱🇾 Libye</option>
<option value="Soudan">🇸🇩 Soudan</option>
<option value="Centrafrique">🇨🇫 Centrafrique</option>
<option value="Somalie">🇸🇴 Somalie</option>
<option value="Djibouti">🇩🇯 Djibouti</option>
<option value="Erythree">🇪🇷 Erythree</option>
<option value="Ouganda">🇺🇬 Ouganda</option>
<option value="Burundi">🇧🇮 Burundi</option>
<option value="Soudan du Sud">🇸🇸 Soudan du Sud</option>
<option value="Zambie">🇿🇲 Zambie</option>
<option value="Zimbabwe">🇿🇼 Zimbabwe</option>
<option value="Mozambique">🇲🇿 Mozambique</option>
<option value="Malawi">🇲🇼 Malawi</option>
<option value="Madagascar">🇲🇬 Madagascar</option>
<option value="Maurice">🇲🇺 Maurice</option>
<option value="Comores">🇰🇲 Comores</option>
<option value="Angola">🇦🇴 Angola</option>
<option value="Namibie">🇳🇦 Namibie</option>
<option value="Botswana">🇧🇼 Botswana</option>
<option value="Lesotho">🇱🇸 Lesotho</option>
<option value="Eswatini">🇸🇿 Eswatini</option>
<option value="Gambie">🇬🇲 Gambie</option>
<option value="Bissau">🇬🇼 Guinee-Bissau</option>
<option value="Sierra Leone">🇸🇱 Sierra Leone</option>
<option value="Liberia">🇱🇷 Liberia</option>
<option value="Cap Vert">🇨🇻 Cap Vert</option>
<option value="Sao Tome">🇸🇹 Sao Tome</option>
<option value="Tunisie">🇹🇳 Tunisie</option>
<option value="Algerie">🇩🇿 Algerie</option>
<option value="Maroc">🇲🇦 Maroc</option>
<option value="Autre">🌍 Autre</option>
</select>
<textarea name="bio" placeholder="Bio (optionnel)" rows="3"></textarea>
<button type="submit">Creer mon compte</button>
</form>
<p style="margin-top:15px;"><a href="/login">Deja inscrit? Connexion</a></p>
</div>
"#;
    html_page("S'inscrire", body)
}

fn html_login() -> String {
    let body = r#"
<div style="max-width:400px;margin:0 auto;">
<h1>Connexion</h1>
<form method="POST" action="/login">
<input type="text" name="username" placeholder="Nom d'utilisateur" required>
<input type="password" name="password" placeholder="Mot de passe" required>
<button type="submit">Se connecter</button>
</form>
<p style="margin-top:15px;"><a href="/register">Pas encore inscrit? S'inscrire</a></p>
</div>
"#;
    html_page("Connexion", body)
}

fn html_new_repo() -> String {
    let body = r#"
<div style="max-width:500px;margin:0 auto;">
<h1>Nouveau depot</h1>
<form method="POST" action="/new">
<input type="text" name="name" placeholder="Nom du depot (ex: mon-projet)" required>
<input type="text" name="description" placeholder="Description courte">
<select name="language">
<option value="Rust">Rust</option>
<option value="Python">Python</option>
<option value="JavaScript">JavaScript</option>
<option value="Shell">Shell</option>
<option value="HTML">HTML</option>
<option value="C">C</option>
<option value="Autre">Autre</option>
</select>
<select name="visibility">
<option value="public">Public — tout le monde peut voir</option>
<option value="private">Prive — seulement toi</option>
</select>
<input type="text" name="tags" placeholder="Tags (separe par virgules: rust, blockchain, africa)">
<button type="submit">Creer le depot</button>
</form>
</div>
"#;
    html_page("Nouveau depot", body)
}

fn html_repo_view(repo: &Repository, owner: &str, name: &str, is_owner: bool, current_user_opt: Option<&str>, comments_html: &str, issues_html: &str) -> String {
    // Historique des commits (v0.20)
    let commits_html = if repo.commits.is_empty() {
        String::new()
    } else {
        let mut list = String::new();
        for cm in repo.commits.iter().rev().take(20) {
            list.push_str(&format!(
                r#"<li style="padding:6px 0;border-bottom:1px solid #21262d;"><span class="badge" style="background:#238636;color:#fff;">✔</span> <strong style="color:#58a6ff;">{}</strong> — {} <span style="color:#8b949e;font-size:0.85em;">par {} · {}</span></li>"#,
                escape_json(&cm.filename), escape_json(&cm.message), escape_json(&cm.author), ts_lisible(&cm.created_at)
            ));
        }
        format!(r#"<div class="card"><h2>📜 Historique des commits ({})</h2><ul style="list-style:none;padding:0;margin:0;">{}</ul></div>"#,
            repo.commits.len(), list)
    };

    let files_html = if repo.files.is_empty() {
        if is_owner {
            format!("<div class='empty'>Aucun fichier. <a href='/{}/{}/upload'>Ajouter un fichier</a></div>", owner, name)
        } else {
            "<div class='empty'>Aucun fichier dans ce depot.</div>".to_string()
        }
    } else {
        let mut files_list = String::new();
        for (fname, content) in &repo.files {
            let lines = content.lines().count();
            files_list.push_str(&format!(
                r#"<li><a href="/{}/{}/file/{}">📄 {}</a> <span style="color:#484f58;float:right;">{} lignes</span></li>"#,
                owner, name, fname, fname, lines
            ));
        }
        format!("<h2>Fichiers</h2><ul class='file-list'>{}</ul>", files_list)
    };

    // README rendering
    let readme_html = if let Some(readme_content) = repo.files.get("README.md")
        .or_else(|| repo.files.get("readme.md"))
        .or_else(|| repo.files.get("README"))
    {
        format!(r#"<div class="card"><h2>📖 README</h2><div style="white-space:pre-wrap;color:#c9d1d9;">{}</div></div>"#, readme_content)
    } else {
        String::new()
    };

    // Tags display
    let tags_html = if repo.tags.is_empty() {
        String::new()
    } else {
        let badges: String = repo.tags.iter().map(|t| {
            format!(r#"<span class="badge" style="background:#1f6feb;color:#fff;">🏷️ {}</span>"#, escape_json(t))
        }).collect::<Vec<_>>().join(" ");
        format!(r#"<div style="margin:5px 0;">{}</div>"#, badges)
    };

    let owner_actions = if is_owner {
        format!(r#"<a href="/{}/{}/upload" class="btn btn-secondary">+ Ajouter fichier</a> <a href="/{}/{}/settings" class="btn btn-secondary">⚙ Parametres</a>"#, owner, name, owner, name)
    } else {
        String::new()
    };

    let star_form = if let Some(_user) = current_user_opt {
        format!(r#"<form method="POST" action="/{}/{}/star" style="display:inline;"><button type="submit" class="btn btn-secondary">🌳 Baobab</button></form>"#, owner, name)
    } else {
        String::new()
    };

    let download_btn = format!(r#"<a href="/{}/{}/download" class="btn btn-secondary" style="display:inline;">⬇ Telecharger ZIP</a>"#, owner, name);

    let fork_form = if let Some(user) = current_user_opt {
        if user != owner {
            format!(r#"<form method="POST" action="/{}/{}/fork" style="display:inline;"><button type="submit" class="btn btn-secondary">🌿 Bouture</button></form>"#, owner, name)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let body = format!(r#"
<div style="display:flex;justify-content:space-between;align-items:center;">
<div>
<h1>{}/<span style="color:#58a6ff;">{}</span></h1>
<p style="color:#8b949e;">{}</p>
{}
</div>
<div>
<span class="badge badge-{}">{}</span>
{} <span class="badge {}">{}</span> {} {} {}
</div>
</div>
<div class="stats">
<div class="stat"><div class="num">⭐ {}</div><div class="label">Baobabs</div></div>
<div class="stat"><div class="num">🍴 {}</div><div class="label">Boutures</div></div>
<div class="stat"><div class="num">{}</div><div class="label">Fichiers</div></div>
<div class="stat"><div class="num">👁 {}</div><div class="label">Vues</div></div>
</div>
<div style="margin:10px 0;">
<a href="/course/{}/{}" class="btn" style="background:#f59e0b;">🎓 Cours & Diplome</a>
</div>
{}
{}
{}
{}
{}
{}
"#, owner, name, repo.description, tags_html,
    if repo.language == "Rust" { "rust" } else if repo.language == "Python" { "python" } else { "js" },
    repo.language,
    owner_actions,
    if repo.is_public { "badge-public" } else { "badge-private" },
    if repo.is_public { "Public" } else { "Prive" },
    star_form, fork_form, download_btn,
    repo.stars, repo.forks, repo.files.len(), repo.views,
    owner, name, tags_html, readme_html, files_html, commits_html, comments_html, issues_html
    );

    html_page(&format!("{}/{}", owner, name), &body)
}

fn html_file_view(repo: &Repository, owner: &str, name: &str, filename: &str, content: &str, is_owner: bool) -> String {
    let delete_btn = if is_owner {
        format!(r#"<form method="POST" action="/{}/{}/delete/{}" style="display:inline;float:right;"><button type="submit" class="btn btn-danger" onclick="return confirm('Supprimer ce fichier?')">Supprimer</button></form>"#, owner, name, filename)
    } else {
        String::new()
    };

    let body = format!(r#"
<div>
<div style="display:flex;justify-content:space-between;align-items:center;">
<h2><a href="/{}/{}">{}/{}</a> / <span style="color:#58a6ff;">{}</span></h2>
{}
</div>
<div class="code-block">{}</div>
</div>
"#, owner, name, owner, name, filename, delete_btn, content);

    html_page(&format!("{}/{}/{}", owner, name, filename), &body)
}

fn html_upload(owner: &str, name: &str) -> String {
    let body = format!(r#"
<div style="max-width:600px;margin:0 auto;">
<h1>Ajouter un fichier a {}/{}</h1>
<form method="POST" action="/{}/{}/upload">
<input type="text" name="filename" placeholder="Nom du fichier (ex: main.rs)" required>
<input type="text" name="commit_message" placeholder="Message du commit (ex: fix: correction du bug)" style="margin:8px 0;">
<textarea name="content" placeholder="Contenu du fichier" rows="15" required></textarea>
<button type="submit">Commiter le fichier</button>
</form>
</div>
"#, owner, name, owner, name);
    html_page("Ajouter un fichier", &body)
}

fn html_user_profile(user: &User, state: &AppState, current_user: Option<&str>) -> String {
    let is_own = current_user.map(|u| u == user.username).unwrap_or(false);
    let user_repos: Vec<&Repository> = state.repos.iter()
        .filter(|r| r.owner == user.username && (r.is_public || is_own))
        .collect();

    let repos_html = if user_repos.is_empty() {
        "<div class='empty'>Aucun depot public.</div>".to_string()
    } else {
        user_repos.iter().map(|r| format!(
            r#"<div class="repo"><h3><a href="/{}/{}">{}/{}</a> <span class="badge badge-{}">{}</span></h3><div class="desc">{}</div><div class="meta">⭐ {} · {}</div></div>"#,
            r.owner, r.name, r.owner, r.name,
            if r.language == "Rust" { "rust" } else if r.language == "Python" { "python" } else { "js" },
            r.language, r.description, r.stars, ts_lisible(&r.created_at)
        )).collect::<Vec<_>>().join("")
    };

    let diplomas = state.get_user_diplomas(&user.username);
    let diplomas_html = if diplomas.is_empty() {
        "<div class='empty'>Aucun diplome encore. Complete des cours pour obtenir des diplomes!</div>".to_string()
    } else {
        diplomas.iter().map(|(name, repo, ex_count)| format!(
            r#"<div class="card" style="border:1px solid #f59e0b;background:#1a1500;">
            <div style="font-size:2em;">🎓</div>
            <h3 style="color:#f59e0b;">{}</h3>
            <p style="color:#8b949e;">Projet: {} | {} exercices</p>
            </div>"#,
            name, repo, ex_count
        )).collect::<Vec<_>>().join("")
    };

    let edit_bio = if is_own {
        format!(r#"<form method="POST" action="/profile/edit" style="margin-top:10px;">
        <textarea name="bio" placeholder="Ta bio..." rows="2">{}</textarea>
        <button type="submit" class="btn btn-secondary">Modifier bio</button>
        </form>"#, user.bio)
    } else {
        String::new()
    };

    let follow_btn = if let Some(cu) = current_user {
        if cu != user.username {
            let is_following = state.is_following(cu, &user.username);
            format!(r#"<form method="POST" action="/follow/{}" style="display:inline;"><button type="submit" class="btn {}">{}</button></form>"#,
                user.username,
                if is_following { "btn-secondary" } else { "" },
                if is_following { "🤝 Sananku" } else { "🤝 + Sananku" }
            )
        } else { String::new() }
    } else { String::new() };

    let follower_count = state.get_followers(&user.username).len();
    let following_count = state.get_following(&user.username).len();

    let total_stars: usize = user_repos.iter().map(|r| r.stars).sum();

    // TROPHEES DU VILLAGE — badges africains calcules depuis l'activite reelle
    let user_repo_ids: Vec<usize> = user_repos.iter().map(|r| r.id).collect();
    let mut trophies: Vec<String> = Vec::new();
    if !user_repos.is_empty() { trophies.push("🏕️ <strong>Premiere Case</strong> — a construit son premier depot".to_string()); }
    if user_repos.len() >= 3 { trophies.push("🏘️ <strong> Chef de Concession</strong> — 3 depots ou plus".to_string()); }
    if total_stars >= 5 { trophies.push("🌳 <strong>Grand Baobab</strong> — 5 baobabs recus".to_string()); }
    if user_repos.iter().any(|r| r.name.starts_with("fork-")) { trophies.push("🌿 <strong>Pepinieriste</strong> — a fait une bouture".to_string()); }
    if state.issues.iter().any(|i| i.author == user.username) { trophies.push("🐜 <strong>Vigilant</strong> — a signale un termite".to_string()); }
    if state.comments.iter().any(|c| c.author == user.username) { trophies.push("💬 <strong>Griot du Dialogue</strong> — a participe aux discussions".to_string()); }
    if follower_count >= 3 { trophies.push("🤝 <strong>Sanankuya</strong> — 3 sanankus ou plus".to_string()); }
    if state.courses.iter().any(|c| user_repo_ids.contains(&c.repo_id)) { trophies.push("🎓 <strong>Professeur</strong> — son depot est devenu cours".to_string()); }
    if diplomas.len() >= 1 { trophies.push("📜 <strong>Eleve Model</strong> — a obtenu un diplome".to_string()); }
    // Diplomes de l'Ecole du Village
    let ecole_done = state.ecole_progress.get(&user.username).cloned().unwrap_or_default();
    if ecole_cycle_complete(&ecole_done, "Semence") { trophies.push("🌱 <strong>Diplome de la Semence</strong> — Ecole du Village, primaire complete".to_string()); }
    if ecole_cycle_complete(&ecole_done, "Griot") { trophies.push("📖 <strong>Diplome du Griot</strong> — Ecole du Village, college complete".to_string()); }
    if ecole_cycle_complete(&ecole_done, "Baobab") { trophies.push("🌳 <strong>Diplome du Baobab</strong> — Ecole du Village, lycee complete".to_string()); }
    if ecole_cycle_complete(&ecole_done, "Sage") { trophies.push("🎓 <strong>Diplome du Sage</strong> — Ecole du Village, universite complete".to_string()); }
    if ecole_cycle_complete(&ecole_done, "Science") { trophies.push("🔬 <strong>Diplome du Savant</strong> — Faculte des Sciences complete".to_string()); }
    if ecole_cycle_complete(&ecole_done, "Maths") { trophies.push("➗ <strong>Diplome du Calculateur</strong> — Faculte des Mathematiques complete".to_string()); }
    if ecole_cycle_complete(&ecole_done, "Techno") { trophies.push("⚙️ <strong>Diplome de l'Ingenieur</strong> — Faculte de Technologie complete".to_string()); }
    // Jeux du Village
    let game = state.game_scores.get(&user.username).cloned().unwrap_or(GameScore { coins: 0, best: 0 });
    if game.best > 0 { trophies.push(format!("🦁 <strong>Chasseur du Sahel</strong> — meilleur score {} au Lion du Sahel ({} pieces au compte)", game.best, game.coins)); }
    let trophies_html = if trophies.is_empty() {
        "<div class='empty'>Aucun trophee encore. Plante ton premier depot!</div>".to_string()
    } else {
        format!("<ul style='margin:0;padding-left:20px;'>{}</ul>", trophies.iter().map(|t| format!("<li style='margin:6px 0;'>{}</li>", t)).collect::<Vec<_>>().join(""))
    };

    let body = format!(r#"
<div style="display:flex;justify-content:space-between;align-items:center;">
<div>
<h1>👤 {}</h1>
<p style="color:#8b949e;">🌍 {} | 📅 Inscription: {}</p>
<p style="margin:10px 0;color:#c9d1d9;">{}</p>
{}
</div>
<div class="stats">
<div class="stat"><div class="num">{}</div><div class="label">Depots</div></div>
<div class="stat"><div class="num">{}</div><div class="label">Diplomes</div></div>
<div class="stat"><div class="num">⭐ {}</div><div class="label">Baobabs recus</div></div>
<div class="stat"><div class="num">{}</div><div class="label">Sanankus</div></div>
<div class="stat"><div class="num">{}</div><div class="label">Mes Sanankus</div></div>
<div class="stat"><div class="num">🪙 {}</div><div class="label">Pieces du Lion</div></div>
</div>
<div style="margin:10px 0;">{}</div>
</div>
<h2>🏅 Trophees du Village</h2>
{}
<h2>📦 Depots</h2>
{}
<h2>🎓 Diplomes</h2>
{}
"#, user.username, user.country, user.created_at,
    if user.bio.is_empty() { "Pas de bio encore." } else { &user.bio },
    edit_bio,
    user_repos.len(), diplomas.len(), total_stars, follower_count, following_count, game.coins,
    follow_btn, trophies_html, repos_html, diplomas_html);

    html_page(&format!("Profil: {}", user.username), &body)
}

fn html_course_catalog(state: &AppState, current_user: Option<&str>) -> String {
    let courses_html = if state.courses.is_empty() {
        "<div class='empty'>Aucun cours disponible encore. Cree un depot et ajoute du code pour generer un cours!</div>".to_string()
    } else {
        state.courses.iter().filter_map(|c| {
            if let Some(repo) = state.repos.iter().find(|r| r.id == c.repo_id) {
                if !repo.is_public && current_user != Some(repo.owner.as_str()) {
                    return None;
                }
                let ex_count = c.exercises.len();
                let mod_count = c.modules.len();
                Some(format!(
                    r#"<div class="card"><h3><a href="/course/{}/{}">🎓 {}</a></h3><p style="color:#8b949e;">{}</p><div class="meta">📚 {} modules · ✏️ {} exercices · 🏷️ {} · Par <a href="/{}">{}</a></div></div>"#,
                    repo.owner, repo.name, c.title, c.description, mod_count, ex_count, repo.language, repo.owner, repo.owner
                ))
            } else {
                None
            }
        }).collect::<Vec<_>>().join("")
    };

    let body = format!(r#"
<h1>🎓 Catalogue de Cours</h1>
<p style="color:#8b949e;">Apprends a coder avec des cours auto-generees a partir de vrais projets africains</p>
{}
"#, courses_html);

    html_page("Catalogue de Cours", &body)
}

fn html_leaderboard(state: &AppState) -> String {
    // Calculate scores: stars received + diplomas earned
    let mut user_scores: Vec<(String, String, usize, usize, usize)> = Vec::new();
    // (username, country, repos_count, diplomas_count, stars_received)

    for user in &state.users {
        let user_repos: Vec<&Repository> = state.repos.iter()
            .filter(|r| r.owner == user.username && r.is_public).collect();
        let repos_count = user_repos.len();
        let stars: usize = user_repos.iter().map(|r| r.stars).sum();
        let diplomas = state.get_user_diplomas(&user.username);
        user_scores.push((user.username.clone(), user.country.clone(), repos_count, diplomas.len(), stars));
    }

    // Sort by total score (diplomas * 3 + stars + repos)
    user_scores.sort_by(|a, b| {
        let score_a = a.3 * 3 + a.4 + a.2;
        let score_b = b.3 * 3 + b.4 + b.2;
        score_b.cmp(&score_a)
    });

    let leaderboard_html = if user_scores.is_empty() {
        "<div class='empty'>Aucun developpeur inscrit encore.</div>".to_string()
    } else {
        let medals = ["🥇", "🥈", "🥉"];
        user_scores.iter().enumerate().map(|(i, (username, country, repos, diplomas, stars))| {
            let medal: String = if i < 3 { medals[i].to_string() } else { format!("{}.", i + 1) };
            format!(
                r#"<div class="card" style="display:flex;justify-content:space-between;align-items:center;">
                <div><span style="font-size:1.5em;">{}</span> <a href="/{}"><strong>{}</strong></a> <span style="color:#8b949e;">🌍 {}</span></div>
                <div class="stats" style="gap:10px;">
                <div class="stat"><div class="num">{}</div><div class="label">Depots</div></div>
                <div class="stat"><div class="num">🎓 {}</div><div class="label">Diplomes</div></div>
                <div class="stat"><div class="num">⭐ {}</div><div class="label">Baobabs</div></div>
                </div>
                </div>"#,
                medal, username, username, country, repos, diplomas, stars
            )
        }).collect::<Vec<_>>().join("")
    };

    let body = format!(r#"
<h1>🏛️ Conseil des Sages — les developpeurs</h1>
<p style="color:#8b949e;">Les meilleurs developpeurs africains — classe par diplomes, stars et depots</p>
{}
"#, leaderboard_html);

    html_page("Conseil des Sages", &body)
}

fn html_search(state: &AppState, current_user: Option<&str>, query: &str) -> String {
    let q = query.to_lowercase();
    let results: Vec<&Repository> = state.repos.iter()
        .filter(|r| {
            r.is_public || current_user == Some(r.owner.as_str())
        })
        .filter(|r| {
            r.name.to_lowercase().contains(&q)
            || r.description.to_lowercase().contains(&q)
            || r.owner.to_lowercase().contains(&q)
            || r.language.to_lowercase().contains(&q)
        })
        .collect();

    let results_html = if results.is_empty() {
        if query.is_empty() {
            "<div class='empty'>Tape quelque chose pour rechercher!</div>".to_string()
        } else {
            format!("<div class='empty'>Aucun resultat pour '{}'</div>", query)
        }
    } else {
        results.iter().map(|r| format!(
            r#"<div class="repo"><h3><a href="/{}/{}">{}/{}</a> <span class="badge badge-{}">{}</span></h3><div class="desc">{}</div><div class="meta">⭐ {} · {}</div></div>"#,
            r.owner, r.name, r.owner, r.name,
            if r.language == "Rust" { "rust" } else if r.language == "Python" { "python" } else { "js" },
            r.language, r.description, r.stars, ts_lisible(&r.created_at)
        )).collect::<Vec<_>>().join("")
    };

    let body = format!(r#"
<h1>🔍 Rechercher</h1>
<form method="GET" action="/search">
<input type="text" name="q" placeholder="Nom, description, langage, ou createeur..." value="{}">
<button type="submit">Rechercher</button>
</form>
<h2>Resultats ({})</h2>
{}
"#, query, results.len(), results_html);

    html_page("Rechercher", &body)
}

fn parse_comments(data: &str) -> Vec<Comment> {
    let mut comments = Vec::new();
    let data = data.trim();
    if data == "[]" || data.is_empty() {
        return comments;
    }
    // Simple parser: find {"repo_id":N,"author":"...","text":"...","created_at":"..."}
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in data.char_indices() {
        if ch == '{' {
            if depth == 0 {
                start = i;
            }
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                let obj = &data[start..=i];
                let repo_id = extract_json_num(obj, "repo_id").map(|n| n as usize).unwrap_or(0);
                let author = extract_json_str(obj, "author").unwrap_or_default();
                let text = extract_json_str(obj, "text").unwrap_or_default();
                let created_at = extract_json_str(obj, "created_at").unwrap_or_default();
                comments.push(Comment {
                    repo_id,
                    author,
                    text,
                    created_at,
                });
            }
        }
    }
    comments
}

fn parse_notifications(data: &str) -> Vec<Notification> {
    let mut notifications = Vec::new();
    let data = data.trim();
    if data == "[]" || data.is_empty() {
        return notifications;
    }
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in data.char_indices() {
        if ch == '{' {
            if depth == 0 { start = i; }
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                let obj = &data[start..=i];
                let username = extract_json_str(obj, "username").unwrap_or_default();
                let message = extract_json_str(obj, "message").unwrap_or_default();
                let link = extract_json_str(obj, "link").unwrap_or_default();
                let created_at = extract_json_str(obj, "created_at").unwrap_or_default();
                let read = extract_json_bool(obj, "read").unwrap_or(false);
                notifications.push(Notification { username, message, link, created_at, read });
            }
        }
    }
    notifications
}

fn parse_issues(data: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let data = data.trim();
    if data == "[]" || data.is_empty() { return issues; }
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in data.char_indices() {
        if ch == '{' {
            if depth == 0 { start = i; }
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                let obj = &data[start..=i];
                issues.push(Issue {
                    id: extract_json_num(obj, "id").unwrap_or(0.0) as usize,
                    repo_id: extract_json_num(obj, "repo_id").unwrap_or(0.0) as usize,
                    title: extract_json_str(obj, "title").unwrap_or_default(),
                    body: extract_json_str(obj, "body").unwrap_or_default(),
                    author: extract_json_str(obj, "author").unwrap_or_default(),
                    created_at: extract_json_str(obj, "created_at").unwrap_or_default(),
                    status: extract_json_str(obj, "status").unwrap_or_default(),
                });
            }
        }
    }
    issues
}

fn parse_follows(data: &str) -> HashMap<String, Vec<String>> {    let mut follows = HashMap::new();
    let data = data.trim();
    if data == "{}" || data.is_empty() { return follows; }
    let mut pos = 0;
    let bytes = data.as_bytes();
    while pos < bytes.len() {
        if bytes[pos] == b'"' {
            let key_start = pos + 1;
            let mut key_end = key_start;
            while key_end < bytes.len() && bytes[key_end] != b'"' { key_end += 1; }
            let key = unescape_json(&data[key_start..key_end]);
            pos = key_end + 1;
            while pos < bytes.len() && bytes[pos] != b'[' { pos += 1; }
            pos += 1;
            let mut users = Vec::new();
            while pos < bytes.len() && bytes[pos] != b']' {
                if bytes[pos] == b'"' {
                    let ustart = pos + 1;
                    let mut uend = ustart;
                    while uend < bytes.len() && bytes[uend] != b'"' {
                        if bytes[uend] == b'\\' { uend += 2; continue; }
                        uend += 1;
                    }
                    users.push(unescape_json(&data[ustart..uend]));
                    pos = uend + 1;
                } else { pos += 1; }
            }
            follows.insert(key, users);
        } else { pos += 1; }
    }
    follows
}

fn parse_starred(data: &str) -> HashMap<String, Vec<usize>> {
    let mut starred: HashMap<String, Vec<usize>> = HashMap::new();
    let data = data.trim();
    if data == "{}" || data.is_empty() { return starred; }
    let mut pos = 0;
    let bytes = data.as_bytes();
    while pos < bytes.len() {
        if bytes[pos] == b'"' {
            let key_start = pos + 1;
            let mut key_end = key_start;
            while key_end < bytes.len() && bytes[key_end] != b'"' { key_end += 1; }
            let key = unescape_json(&data[key_start..key_end]);
            pos = key_end + 1;
            while pos < bytes.len() && bytes[pos] != b'[' { pos += 1; }
            pos += 1;
            let mut ids = Vec::new();
            let mut num = String::new();
            while pos < bytes.len() && bytes[pos] != b']' {
                let c = bytes[pos] as char;
                if c.is_ascii_digit() {
                    num.push(c);
                } else if !num.is_empty() {
                    if let Ok(id) = num.parse::<usize>() { ids.push(id); }
                    num.clear();
                }
                pos += 1;
            }
            if !num.is_empty() {
                if let Ok(id) = num.parse::<usize>() { ids.push(id); }
            }
            starred.insert(key, ids);
        } else { pos += 1; }
    }
    starred
}

fn parse_ecole_progress(data: &str) -> HashMap<String, Vec<String>> {
    let mut progress: HashMap<String, Vec<String>> = HashMap::new();
    let data = data.trim();
    if data == "{}" || data.is_empty() { return progress; }
    let mut pos = 0;
    let bytes = data.as_bytes();
    while pos < bytes.len() {
        if bytes[pos] == b'"' {
            let key_start = pos + 1;
            let mut key_end = key_start;
            while key_end < bytes.len() && bytes[key_end] != b'"' { key_end += 1; }
            let key = unescape_json(&data[key_start..key_end]);
            pos = key_end + 1;
            while pos < bytes.len() && bytes[pos] != b'[' { pos += 1; }
            pos += 1;
            let mut levels = Vec::new();
            while pos < bytes.len() && bytes[pos] != b']' {
                if bytes[pos] == b'"' {
                    let s_start = pos + 1;
                    let mut s_end = s_start;
                    while s_end < bytes.len() && bytes[s_end] != b'"' { s_end += 1; }
                    levels.push(unescape_json(&data[s_start..s_end]));
                    pos = s_end + 1;
                } else { pos += 1; }
            }
            progress.insert(key, levels);
        } else { pos += 1; }
    }
    progress
}

// ============================================================
// ECOLE DU VILLAGE — lecons et exercices (reponses cachees cote serveur)
// ============================================================
struct EcoleLevel {
    slug: &'static str,
    title: &'static str,
    cycle: &'static str,
    lessons: &'static str,
    exercises: Vec<(&'static str, &'static str, &'static str)>, // (question, reponse, explication)
}


fn extract_num_after(seg: &str, marker: &str) -> Option<u32> {
    let pos = seg.find(marker)? + marker.len();
    let sub = &seg[pos..];
    let end = sub.find(|ch: char| !ch.is_ascii_digit()).unwrap_or(sub.len());
    sub[..end].parse().ok()
}

fn parse_game_scores(data: &str) -> HashMap<String, GameScore> {
    let mut scores: HashMap<String, GameScore> = HashMap::new();
    let data = data.trim();
    if data == "{}" || data.is_empty() { return scores; }
    let mut rest = data;
    while let Some(qpos) = rest.find('"') {
        let after = &rest[qpos + 1..];
        let Some(kend) = after.find('"') else { break };
        let key = unescape_json(&after[..kend]);
        let seg = &after[kend + 1..];
        if let (Some(c), Some(b)) = (extract_num_after(seg, "\"coins\":"), extract_num_after(seg, "\"best\":")) {
            scores.insert(key, GameScore { coins: c, best: b });
        }
        match seg.find('}') { Some(p) => { rest = &seg[p + 1..]; } None => break }
    }
    scores
}

fn ecole_levels() -> Vec<EcoleLevel> {
    vec![
        EcoleLevel { slug: "cp1", title: "CP1 — Le Decouvreur", cycle: "Semence",
            lessons: r#"<h3>Lecon 1: Les sons de la nature (N-KCOL)</h3><p>N-KCOL, c'est lire la nature. Chaque lettre est un son vivant: C c'est HOUUU le tourbillon, K c'est TÈK le bois qui previent, X c'est TCHAK qui coupe la nuit. Un aveugle doit comprendre le son, un sourd doit sentir la vibration, un enfant de 3 ans doit pouvoir l'imiter.</p><h3>Lecon 2: Les animaux parlent N-KCOL</h3><p>Le coq ne dit pas "cocorico" — il dit TÈK-ETCHIIII-TCHAK-OHHHH. Le mouton dit M-B-OHHHH. La grenouille dit DRIIIIP-DJRRR. Le hibou dit OHHHH-OUUUH. L'abeille dit MMMMM. Ecoute la nature: elle parle notre langue.</p>"#,
            exercises: vec![
                ("Quel animal dit TÈK-ETCHIIII-TCHAK-OHHHH ?", "coq", "POURQUOI: N-KCOL ecoute le vrai son de la nature, pas le mot francais 'cocorico'. COMMENT: le chant du coq suit 4 lois vivantes — TÈK (il previent), ETCHIIII (il s'etire), TCHAK (il tranche la nuit), OHHHH (il annonce le jour). VOILA: quand tu ecoutes un coq africain, tu entends N-KCOL parler."),
                ("Quel est le son de la lettre C dans N-KCOL ?", "houuu", "POURQUOI: dans N-KCOL, chaque lettre est un son de la nature, pas un symbole mort. COMMENT: le C est le tourbillon — HOUUU. Il passe les 3 lois: un aveugle comprend le son, un sourd sent la vibration, un enfant de 3 ans l'imite. VOILA: C = tourbillon, pour toujours."),
                ("Combien de lettres a l'alphabet N-KCOL ?", "26", "POURQUOI: N-KCOL est le label qui cree toutes les langues du monde. COMMENT: 26 lettres, chacune avec son symbole, son son, son sens — toutes les langues naissent de la meme nature. VOILA: 26 lettres — l'alphabet vivant."),
            ] },
        EcoleLevel { slug: "cp2", title: "CP2 — Le Conteur en Herbe", cycle: "Semence",
            lessons: r#"<h3>Lecon 1: Les contes du village</h3><p>Pourquoi la tortue a une carapace? Parce qu'elle a vole la sagesse du village et a du se cacher dedans. Les contes africains n'amusent pas seulement: ils enseignent. Chaque conte cache une lecon de sagesse.</p><h3>Lecon 2: Compter en langues africaines</h3><p>En bambara: kelen (1), fila (2), saba (3), naani (4), duuru (5). En wolof: benn, jar, yett... Chaque langue africaine a sa propre arithmetique. Compter dans sa langue, c'est penser dans sa langue.</p>"#,
            exercises: vec![
                ("Qui a vole la sagesse dans le conte?", "tortue", "POURQUOI: les contes africains enseignent, ils n'amusent pas seulement. COMMENT: la tortue vole la sagesse du village, les villageois la poursuivent, elle se cache dans sa carapace. VOILA: la sagesse volee ne nourrit pas — elle pese. Le conte donne la lecon."),
                ("Comment dit-on 3 en bambara?", "saba", "POURQUOI: compter dans sa langue, c'est penser dans sa langue. COMMENT: en bambara kelen (1), fila (2), saba (3), naani (4), duuru (5). VOILA: ton cerveau calcule plus vite car il ne traduit pas."),
                ("Comment dit-on 5 en bambara?", "duuru", "POURQUOI: chaque langue africaine a sa propre arithmetique. COMMENT: duuru c'est 5 — la main complete, les cinq doigts du paysan qui seme. VOILA: un mot bambara, un concept universel."),
            ] },
        EcoleLevel { slug: "ce1", title: "CE1 — L'Explorateur", cycle: "Semence",
            lessons: r#"<h3>Lecon 1: Les 54 pays d'Afrique</h3><p>L'Afrique compte 54 pays, du Senegal a la Somalie, du Maroc a l'Afrique du Sud. Chaque drapeau raconte une histoire: le vert du Mali c'est la nature, l'or c'est la richesse du soleil, le rouge c'est le sang des martyrs.</p><h3>Lecon 2: Les grands fleuves</h3><p>Le Nil: le plus long (6650 km), il a nourri l'Egypte ancienne. Le Niger: il traverse le Mali et fait vivre tout le Sahel. Le Congo: le plus puissant, le coeur de la foret. Un fleuve africain, c'est une artere du continent.</p>"#,
            exercises: vec![
                ("Combien de pays a l'Afrique?", "54", "POURQUOI: connaitre son continent, c'est connaitre sa maison. COMMENT: 54 pays reconnus, du Maroc a l'Afrique du Sud, du Senegal aux Seychelles. VOILA: 54 drapeaux, 54 fiertes, une seule Afrique."),
                ("Quel fleuve traverse le Mali?", "niger", "POURQUOI: les fleuves sont les arteres du continent. COMMENT: le Niger nait en Guinee, traverse le Mali (Segou, Tombouctou), le Niger, le Nigeria, et rejoint l'Atlantique. VOILA: il fait vivre tout le Sahel — sans lui pas de riz, pas de mil, pas de villes."),
                ("Quel est le plus long fleuve d'Afrique?", "nil", "POURQUOI: le Nil est le fleuve-mere de la civilisation. COMMENT: 6650 km, le plus long du monde, il traverse 11 pays avant la mer. VOILA: sans le Nil, ni pyramides, ni Egypte ancienne — le fleuve a ecrit l'histoire."),
            ] },
        EcoleLevel { slug: "ce2", title: "CE2 — Le Sage des Betes", cycle: "Semence",
            lessons: r#"<h3>Lecon 1: Les fables africaines</h3><p>Le lievre est petit mais malin. L'hyene est forte mais bete. Dans toutes les fables africaines, l'intelligence bat la force. C'est la lecon du Sahel: la ruse du lievre gagne toujours contre les dents de l'hyene.</p><h3>Lecon 2: Le calendrier agricole</h3><p>Au Sahel, l'annee suit la pluie: la saison seche (octobre-mai) et l'hivernage, la saison des pluies (juin-septembre). Le paysan africain lit le ciel, les oiseaux, les termites — il sait quand semer sans aucune montre.</p>"#,
            exercises: vec![
                ("Quel animal est le plus malin dans les fables africaines?", "lievre", "POURQUOI: la sagesse africaine place l'esprit au-dessus de la force. COMMENT: dans toutes les fables, le lievre petit et malin gagne contre les grands et les forts. VOILA: la ruse du lievre est la lecon du Sahel — l'intelligence bat les dents."),
                ("Comment appelle-t-on la saison des pluies au Sahel?", "hivernage", "POURQUOI: le paysan africain lit la nature, pas la montre. COMMENT: l'annee suit la pluie — saison seche (octobre-mai), hivernage (juin-septembre). Oiseaux, termites, lune annoncent la semaille. VOILA: la nature est le calendrier du Sahel."),
                ("Quel animal est fort mais bete dans les fables?", "hyene", "POURQUOI: les fables opposent deux forces — l'esprit et les muscles. COMMENT: l'hyene est puissante mais bete, elle represente la force sans reflexion. VOILA: le lievre gagne toujours — voila pourquoi les griots racontent."),
            ] },
        EcoleLevel { slug: "cm1", title: "CM1 — L'Historien Junior", cycle: "Semence",
            lessons: r#"<h3>Lecon 1: Les empires du Ghana et du Mali</h3><p>L'empire du Ghana (300-1240): le pays de l'or, la ville de Koumbi Saleh. Puis l'empire du Mali (1235): Soundjata Keita l'a fonde apres la bataille de Kirina contre le roi Soumaoro. Soundjata, le lion du Mali, a transforme un peuple brise en empire.</p><h3>Lecon 2: Mansa Moussa, l'homme le plus riche</h3><p>Mansa Moussa (1312-1337) a fait le pelerinage a La Mecque avec 100 000 hommes et des tonnes d'or. Il a tant donne que l'or a perdu sa valeur au Caire. Les historiens disent: l'homme le plus riche de toute l'histoire — et il etait africain.</p>"#,
            exercises: vec![
                ("Qui a fonde l'empire du Mali?", "soundjata", "POURQUOI: l'empire du Mali est ne d'un enfant brise devenu lion. COMMENT: Soundjata Keita, paralyse enfant, se leve, vainc le roi sorcier Soumaoro a Kirina en 1235. VOILA: un peuple brise devient empire — la plus grande lecon d'Afrique de l'Ouest."),
                ("Quel empereur est l'homme le plus riche de l'histoire?", "mansa moussa", "POURQUOI: la richesse de l'Afrique est historique, pas nouvelle. COMMENT: Mansa Moussa (1312-1337) part en pelerinage avec 100 000 hommes et des tonnes d'or — au Caire, la valeur de l'or chute pendant des annees. VOILA: l'homme le plus riche de l'histoire etait africain."),
                ("Quelle bataille a fonde l'empire du Mali?", "kirina", "POURQUOI: les batailles fondent les empires. COMMENT: Kirina (1235) oppose Soundjata au roi sorcier Soumaoro. Victoire de Soundjata — naissance de l'empire du Mali et de la Charte de Kurukan Fuga. VOILA: une bataille, un empire, une constitution."),
            ] },
        EcoleLevel { slug: "cm2", title: "CM2 — L'Heritier", cycle: "Semence",
            lessons: r#"<h3>Lecon 1: L'empire Songhai et Tombouctou</h3><p>L'empire Songhai (1464-1591): Sonni Ali Ber puis Askia Mohammed. Tombouctou etait l'universite du desert: Ahmed Baba, le grand savant, avait 1600 livres quand les bibliothiques d'Europe en avaient 10. L'Afrique ecrivait quand d'autres ne lisaient pas.</p><h3>Lecon 2: Les mathematiques africaines</h3><p>Les fractales: les motifs du kente, les coiffures tresses, les villages en spirale — les mathematiciens africains utilisaient la geometrie des fractales des siecles avant que l'Occident ne la decouvre. Et les pyramides d'Egypte: calculees avec une precision que nos ingenieurs admirent encore.</p>"#,
            exercises: vec![
                ("Quelle ville etait l'universite du desert?", "tombouctou", "POURQUOI: l'Afrique ecrivait quand d'autres ne lisaient pas. COMMENT: Tombouctou, l'universite du desert — Sankore, des milliers de manuscrits, des savants du monde entier venaient etudier. VOILA: la ville du savoir au milieu du sable."),
                ("Quel savant de Tombouctou avait 1600 livres?", "ahmed baba", "POURQUOI: le savoir africain a ses heros. COMMENT: Ahmed Baba (1556-1627) possedait 1600 livres quand les grandes bibliotheques d'Europe en avaient 10. VOILA: le savant de Tombouctou, symbole du savoir africain."),
                ("Quel empire a suivi le Mali?", "songhai", "POURQUOI: les empires se transmettent le flambeau. COMMENT: Ghana (300-1240), puis Mali (1235), puis Songhai (1464-1591) — Sonni Ali Ber le fonde, Askia Mohammed le rayonne. VOILA: le plus grand empire ouest-africain de l'histoire."),
            ] },
        EcoleLevel { slug: "6eme", title: "6eme — L'Apprenti Tambour", cycle: "Griot",
            lessons: r#"<h3>Lecon 1: Les royaumes d'Afrique</h3><p>Le royaume Ashanti: le tabouret d'or, symbole de l'ame du peuple. Le Dahomey: les Amazones, des guerrieres que l'Europe redoutait. Le Kanem-Bornou: mille ans d'histoire autour du lac Tchad. Le Wassoulou: l'empire de Samori.</p><h3>Lecon 2: Le tambour parleur</h3><p>Le tambour imite la langue: il reproduit les tons des mots. Un message tamboure voyage de village en village plus vite qu'un cavalier. Avant le telephone, l'Afrique avait deja son reseau de communication — le tambour parleur.</p>"#,
            exercises: vec![
                ("Quel royaume avait des guerrieres amazones?", "dahomey", "POURQUOI: les femmes africaines se battaient pour leur royaume. COMMENT: le Dahomey avait un regiment d'Amazones — des guerrieres d'elite que l'Europe redoutait. VOILA: les Amazones du Dahomey, fierte de l'Afrique."),
                ("Quel instrument porte les messages a travers la savane?", "tambour", "POURQUOI: avant le telephone, l'Afrique avait son reseau. COMMENT: le tambour parleur imite les tons de la langue — le message voyage de village en village plus vite qu'un cavalier. VOILA: le premier reseau de communication du continent."),
                ("Quel objet sacre symbolise le royaume Ashanti?", "tabouret", "POURQUOI: les symboles unissent les peuples. COMMENT: le tabouret d'or Ashanti porte l'ame du peuple — il descend du ciel, personne ne s'assoit dessus. VOILA: l'objet le plus sacre du royaume."),
            ] },
        EcoleLevel { slug: "5eme", title: "5eme — L'Apprenti Griot", cycle: "Griot",
            lessons: r#"<h3>Lecon 1: La traite, la verite sans fard</h3><p>Pendant des siecles, des millions d'Africains ont ete deportes vers les Ameriques. L'Afrique a ete saignee de ses enfants. On ne l'oublie pas pour pleurer — on s'en souvient pour ne plus jamais le laisser arriver. La memoire est un bouclier.</p><h3>Lecon 2: Les resistances</h3><p>La reine Aline Sitoe Diatta: la femme qui reveille, la Casamance s'est levee derriere elle (1942). Samori Toure: l'Almamy, 7 ans de resistance aux Francais, son empire du Wassoulou. Behanzin: le requin du Dahomey. Ils ont perdu les batailles, mais ils ont gagne notre fierte.</p>"#,
            exercises: vec![
                ("Quelle reine a resiste en Casamance?", "aline sitoe diatta", "POURQUOI: les resistances africaines ont leurs heros feminins. COMMENT: Aline Sitoe Diatta, la reine de Casamance, a resiste a la France coloniale. Deportee, elle n'a jamais cede. VOILA: la reine qui a dit non."),
                ("Quel Almamy a resiste 7 ans aux Francais?", "samori toure", "POURQUOI: la resistance africaine a dure des decennies. COMMENT: Samori Toure, l'Almamy de l'empire du Wassoulou, a resiste 7 ans aux Francais avec une armee organisee et des fusils fabriques sur place. VOILA: le general qui a presque gagne."),
                ("Quel animal symbolise Behanzin?", "requin", "POURQUOI: les rois africains choisissaient leurs symboles. COMMENT: Behanzin, roi du Dahomey, avait le requin pour symbole — puissant, insaisissable, maitre des eaux. VOILA: le roi requin d'Abomey."),
            ] },
        EcoleLevel { slug: "4eme", title: "4eme — Le Jeune Conscience", cycle: "Griot",
            lessons: r#"<h3>Lecon 1: La colonisation</h3><p>En 1884-85, a la conference de Berlin, l'Europe a partage l'Afrique comme un gateau — sans un seul Africain a la table. Les frontieres ont coupe les peuples: les Touaregs entre 5 pays, les Peuls entre 15. Les cultures ont ete brisees, les langues interdites a l'ecole.</p><h3>Lecon 2: Ubuntu et la Charte de Kurukan Fuga</h3><p>Ubuntu: "Je suis parce que nous sommes" — la philosophie du sud du continent. Et en 1236, apres Kirina, Soundjata a proclame la Charte de Kurukan Fuga: 44 articles oraux — le droit a la vie, la protection de l'environnement, l'interdiction de l'esclavage interne. La premiere constitution du monde, et elle etait africaine.</p>"#,
            exercises: vec![
                ("Dans quelle ville les frontieres africaines ont-elles ete tracees?", "berlin", "POURQUOI: les frontieres actuelles portent une blessure. COMMENT: la conference de Berlin (1884-85) — des Europeens decoupent l'Afrique a la regle, sans aucun Africain a la table. VOILA: voila pourquoi des familles se retrouvent de deux cotes d'une meme ligne."),
                ("Complete Ubuntu: 'Je suis parce que...'", "nous sommes", "POURQUOI: l'Afrique a sa propre philosophie de la personne. COMMENT: Ubuntu — 'Je suis parce que nous sommes.' L'individu n'existe que par la communaute. VOILA: le contraire du 'chacun pour soi' occidental."),
                ("Quelle charte de 1236 est la premiere constitution orale?", "kurukan fuga", "POURQUOI: la constitution orale africaine precede beaucoup de textes occidentaux. COMMENT: la Charte de Kurukan Fuga (1236), proclamee par Soundjata apres Kirina — droits, devoirs, organisation sociale. VOILA: une des premieres declarations des droits de l'humanite."),
            ] },
        EcoleLevel { slug: "3eme", title: "3eme — Le Griot", cycle: "Griot",
            lessons: r#"<h3>Lecon 1: Les independances et les peres fondateurs</h3><p>1960: 17 pays africains deviennent independants en une seule annee. Kwame Nkrumah: le Ghana d'abord, l'unite africaine ensuite — "Seek ye first the political kingdom". Patrice Lumumba: le Congo libre, tue en 1961 pour son petrole. Amilcar Cabral: la liberation de la Guinnee-Bissau, l'arme de la theorie.</p><h3>Lecon 2: La medecine traditionnelle et le temps N-KCOL</h3><p>Le neem: l'arbre qui guerit tout — paludisme, plaies, peau. Le moringa: l'arbre de vie, plus de vitamines qu'aucun legume. Le kinkeliba: le the du Sahel qui purifie. Et le temps N-KCOL: pas 24 heures — 4 passages: Naissance, Vie, Mort, Naissance. Une nuit est une vie entiere.</p>"#,
            exercises: vec![
                ("Combien de pays africains ont eu l'independance en 1960?", "17", "POURQUOI: 1960 est l'annee de l'eclair africain. COMMENT: 17 pays gagnent l'independance en une seule annee — du Senegal au Nigeria, du Mali a Madagascar. VOILA: l'annee ou l'Afrique a dit 'assez' en choeur."),
                ("Quel arbre est appele l'arbre de vie?", "moringa", "POURQUOI: la nature africaine nourrit et guerit. COMMENT: le moringa — feuilles riches en vitamines, proteines, fer. On l'appelle l'arbre de vie car il pousse vite et nourrit tout. VOILA: la pharmacie et la cantine du village."),
                ("Combien de passages a le temps N-KCOL?", "4", "POURQUOI: N-KCOL remplace la montre par la nature. COMMENT: le temps suit 4 passages — Naissance, Vie, Mort, Naissance. Une nuit = une vie, un jour = une vie. VOILA: le cercle du temps, pas la ligne du colon."),
            ] },
        EcoleLevel { slug: "2nde", title: "2nde — Le Jeune Lion", cycle: "Baobab",
            lessons: r#"<h3>Lecon 1: La richesse de l'Afrique</h3><p>30% des minerais du monde sont sous nos pieds. 60% des terres arables non exploitees de la planete sont ici. Le meilleur soleil du monde nous eclaire. Le coltan du Congo est dans chaque telephone de la planete — et l'Afrique ne fixe pas les prix. La richesse est africaine; la decision ne l'est pas encore.</p><h3>Lecon 2: Le FCFA et la ZLECAf</h3><p>Le FCFA: cree par la France en 1945, ancre a l'euro, frappe en Europe. Une monnaie que l'Afrique ne controle pas est une chaine invisible. La ZLECAf: le marche commun des 54 pays — 1,4 milliard d'Africains qui peuvent echanger sans barriers. L'avenir: notre monnaie, notre marche, nos prix.</p>"#,
            exercises: vec![
                ("Quel pourcentage des minerais du monde est en Afrique?", "30", "POURQUOI: la richesse africaine se mesure en chiffres, pas en discours. COMMENT: l'Afrique a 30% des minerais du monde — or, coltan, lithium, uranium. Mais elle fixe 0% des prix. VOILA: le probleme n'est pas la richesse, c'est qui tient le comptoir."),
                ("Qui a cree le FCFA en 1945?", "france", "POURQUOI: le FCFA est une monnaie coloniale qui a survecu a la colonisation. COMMENT: cree par la France en 1945 pour ses colonies — meme aujourd'hui, 50% des reserves restent au Tresor francais. VOILA: voila pourquoi l'AES veut sa propre monnaie."),
                ("Quel marche commun reunit les 54 pays africains?", "zlecaf", "POURQUOI: l'unite economique est la vraie force. COMMENT: la ZLECAf — Zone de Libre-Echange Continentale Africaine — 54 pays, 1,3 milliard de consommateurs, le plus grand marche du monde. VOILA: l'Afrique qui commerce avec l'Afrique."),
            ] },
        EcoleLevel { slug: "1ere", title: "1ere — Le Batisseur", cycle: "Baobab",
            lessons: r#"<h3>Lecon 1: Le code, la blockchain</h3><p>Rust compile sur Termux: on peut coder sur son telephone, sans dependance, sans permission. La blockchain: un grand livre que personne ne peut effacer ni falsifier — chaque bloc porte le sceau du precedent. Ed25519: la signature cryptographique qui prouve que c'est toi, sans reveler ton secret. La souverainete numerique s'ecrit en code.</p><h3>Lecon 2: Le soleil serveur</h3><p>L'Afrique a le meilleur gisement solaire de la planete: 6,8 kWh/m2/jour au Niger. Chaque village peut avoir son energie sans reseau occidental. Le soleil est gratuit, il est a nous, il ne demande pas de permission. L'energie est la nouvelle souverainete.</p>"#,
            exercises: vec![
                ("Quel langage compile sur Termux sans dependance?", "rust", "POURQUOI: le code souverain doit compiler partout sans maitre etranger. COMMENT: Rust — zero dependance possible, Cargo.toml [dependencies] vide, compile sur Termux, sur Linux, partout. VOILA: le langage qui a construit AfriChain sur un telephone."),
                ("Quel grand livre personne ne peut effacer?", "blockchain", "POURQUOI: l'Afrique a besoin d'un livre que personne ne peut effacer. COMMENT: la blockchain — chaque bloc porte le hash du precedent, modifier un bloc casse toute la chaine. VOILA: le grand livre que le colon ne peut plus bruler."),
                ("Quelle source d'energie rend l'Afrique souveraine?", "soleil", "POURQUOI: l'energie est la cle de la souverainete. COMMENT: l'Afrique a le meilleur ensoleillement du monde — 6 kWh/m2/jour au Niger. Le soleil est gratuit, abondant, africain. VOILA: le serveur que personne ne peut couper."),
            ] },
        EcoleLevel { slug: "terminale", title: "Terminale — L'Aine", cycle: "Baobab",
            lessons: r#"<h3>Lecon 1: Le leadership africain</h3><p>Thomas Sankara: 4 ans au Burkina (1983-87) — 2,5 millions de vaccins, des ecoles pour les enfants, des femmes au gouvernement. "La patrie ou la mort, nous vaincrons." Aujourd'hui l'AES: le Mali, le Niger, le Burkina sortent du FCFA et construisent leur confederation. Le leadership africain ne demande pas — il construit.</p><h3>Lecon 2: L'union et l'avenir</h3><p>Nkrumah reve des Etats-Unis d'Afrique: un continent, une voix. N-KCOL devient notre langage de programmation souverain. L'Afrique de 2050: 2,5 milliards d'habitants, la plus jeune population du monde, la technologie entre ses mains. Le baton de l'humanite revient a qui l'a fait naitre: l'Afrique guide l'humanite.</p>"#,
            exercises: vec![
                ("Quel president du Burkina a vaccine 2,5 millions d'enfants?", "sankara", "POURQUOI: le leadership africain se mesure aux actes. COMMENT: Thomas Sankara, 4 ans au Burkina (1983-87) — 2,5 millions de vaccines, ecoles, femmes au gouvernement, salaires des ministres coupes. VOILA: 'La patrie ou la mort, nous vaincrons.'"),
                ("Quelle alliance reunit le Mali, le Niger et le Burkina?", "aes", "POURQUOI: la souverainete se construit en alliance. COMMENT: l'AES — Alliance des Etats du Sahel — Mali, Niger, Burkina Faso. Sortie du FCFA en projet, confederation en construction. VOILA: la premiere confederation souveraine de l'Afrique moderne."),
                ("Quel reve de Nkrumah doit unir l'Afrique?", "etats-unis d'afrique", "POURQUOI: le reve de Nkrumah reste le cap. COMMENT: Kwame Nkrumah, pere de l'independance du Ghana, reveait des Etats-Unis d'Afrique — un continent, une monnaie, une armee, une voix. VOILA: l'union totale, l'objectif final."),
            ] },
        EcoleLevel { slug: "l1", title: "Licence 1 — L'Initie", cycle: "Sage",
            lessons: r#"<h3>Lecon 1: La blockchain souveraine</h3><p>AfriChain, c'est Rust std only — zero dependance. Ed25519 pour les signatures, AfriHash-256 pour le minage. Chaque bloc contient: index, timestamp, transactions, prev_hash, nonce. Le minage trouve un nonce tel que hash < difficulte. Pas de Bitcoin, pas d'Ethereum — notre propre chaine, notre propre monnaie.</p><h3>Lecon 2: L'economie africaine</h3><p>Le FCFA: 50% des reserves dans le Tresor francais. La ZLECAf: 1,3 milliard de consommateurs, le plus grand marche du monde. L'AES sort du FCFA et cree sa propre monnaie. L'Afrique a 30% des minerais du monde mais fixe 0% des prix. L'economie souveraine, c'est controler la production, la transformation et la vente.</p>"#,
            exercises: vec![
                ("Quel algorithme AfriChain utilise pour les signatures?", "ed25519", "POURQUOI: les signatures prouvent qui parle sans reveler le secret. COMMENT: Ed25519 — ta cle privee signe, ta cle publique verifie. AfriChain l'a implemente from scratch, zero dependance. VOILA: la signature souveraine, ecrite par nous."),
                ("Quel pourcentage des reserves FCFA est en France?", "50", "POURQUOI: une monnaie dont les reserves sont ailleurs n'est pas une monnaie. COMMENT: le FCFA — 50% des reserves au Tresor francais, par accord colonial toujours valable. VOILA: voila pourquoi chaque franc CFA est un bail colonial."),
                ("Combien de consommateurs a la ZLECAf?", "1,3 milliard", "POURQUOI: le marche interieur africain est le plus grand du monde. COMMENT: la ZLECAf reunit 54 pays et 1,3 milliard de consommateurs — plus grand que l'UE et les USA reunis. VOILA: produisons pour nous d'abord."),
            ] },
        EcoleLevel { slug: "l2", title: "Licence 2 — Le Batisseur", cycle: "Sage",
            lessons: r#"<h3>Lecon 1: Le code souverain</h3><p>Rust: zero cout, zero dependance, compile partout. Cargo.toml [dependencies] vide. std::net pour le reseau, std::fs pour les fichiers. Le code souverain, c'est quand personne ne peut couper ton acces — tu controlles le compilateur, tu controlles le langage.</p><h3>Lecon 2: Les reseaux mesh</h3><p>AfriMesh: UDP broadcast pour la decouverte, TCP pour les messages. Chaque telephone est un noeud. Pas de tour cellulaire, pas de satellite etranger — les telephones africains communiquent directement entre eux.</p>"#,
            exercises: vec![
                ("Quelle section de Cargo.toml reste vide en code souverain?", "dependencies", "POURQUOI: chaque dependance est une chaine. COMMENT: Cargo.toml [dependencies] vide — tout le code est std only: reseau, fichiers, crypto, JSON, HTTP. VOILA: personne ne peut couper l'acces a ce qui n'est pas emprunte."),
                ("Quel protocole AfriMesh utilise pour la decouverte des noeuds?", "udp", "POURQUOI: la decouverte des noeuds doit marcher sans serveur central. COMMENT: AfriMesh envoie des broadcasts UDP — chaque telephone proche entend 'je suis un noeud'. VOILA: le village se decouvre tout seul."),
                ("Quel protocole AfriMesh utilise pour les messages?", "tcp", "POURQUOI: les messages doivent arriver complets et fiables. COMMENT: une fois decouverts, les noeuds se parlent en TCP — connexion durable, messages signes Ed25519. VOILA: les telephones africains se parlent sans tour, sans satellite etranger."),
            ] },
        EcoleLevel { slug: "l3", title: "Licence 3 — Le Strategie", cycle: "Sage",
            lessons: r#"<h3>Lecon 1: La cryptographie pratique</h3><p>Hash: entree → sponge → sortie. AfriHash-256: 5x5 mots 64 bits, 24 rounds (theta, rho, pi, chi, iota). Avalanche: 1 bit change → 50% des bits changes. Collision: 2 entrees differentes → meme sortie (tres rare). Signature: cle privee signe, cle publique verifie.</p><h3>Lecon 2: L'energie solaire</h3><p>L'Afrique a le meilleur ensoleillement du monde: 6 kWh/m2/jour au Niger. Le PoST (Proof of Solar Time): minage seulement quand le soleil brille. Le Sahara pourrait alimenter toute l'Europe — mais l'Afrique garde son energie pour l'Afrique.</p>"#,
            exercises: vec![
                ("Combien de rounds AfriHash-256 utilise?", "24", "POURQUOI: la securite d'un hash vient de ses tours. COMMENT: AfriHash-256 — construction sponge, 5x5 mots de 64 bits, 24 rounds (theta, rho, pi, chi, iota). VOILA: 24 tours, 51% d'avalanche — notre propre hash, pas SHA."),
                ("Quel pays a le meilleur ensoleillement au monde?", "niger", "POURQUOI: la souverainete energetique commence par la carte du soleil. COMMENT: le Niger recoit 6 kWh/m2/jour — le meilleur ensoleillement du monde. VOILA: le desert qui va alimenter le continent."),
                ("Quel consensus solaire AfriChain utilise?", "post", "POURQUOI: le minage doit suivre la nature africaine. COMMENT: PoST — Proof of Solar Time — on mine quand le soleil brille, chaque pays mine a son tour selon son ensoleillement reel. VOILA: le consensus qui respecte le soleil."),
            ] },
        EcoleLevel { slug: "m1", title: "Master 1 — Le Visionnaire", cycle: "Sage",
            lessons: r#"<h3>Lecon 1: L'IA africaine</h3><p>L'IA occidentale est entrainee sur les donnees africaines volees. Notre IA: donnees africaines, hebergees en Afrique, au service de l'Afrique. Le Griot IA: repond en N-KCOL, connait les 54 pays, enseigne l'histoire africaine. Pas de OpenAI, pas de Claude — notre propre intelligence.</p><h3>Lecon 2: La souverainete numerique</h3><p>Les donnees africaines voyagent par cables sous-marins vers l'Europe. DNS: chaque requete Google = donnee vendue. AfriNet: nos propres serveurs, notre propre DNS, nos propres cables. L'Afrique ne demande plus la permission — elle construit.</p>"#,
            exercises: vec![
                ("Quelle IA AfriForme utilise pour enseigner?", "griot", "POURQUOI: l'IA africaine doit connaitre l'Afrique d'abord. COMMENT: Le Griot — l'IA d'AfriForme — enseigne, evalue, explique. Pas de OpenAI, pas de serveur etranger. VOILA: notre intelligence, pour nos enfants."),
                ("Par ou voyagent les donnees africaines vers l'Occident?", "cables sous-marins", "POURQUOI: les donnees africaines voyagent souvent par la porte de l'Occident. COMMENT: les cables sous-marins routent les donnees africaines vers l'Europe — chaque clic passe par Londres ou Marseille. VOILA: AfriNet veut nos serveurs, nos routes, nos donnees chez nous."),
                ("Qui controle le DNS occidental?", "google", "POURQUOI: le DNS est l'annuaire d'internet — qui le tient vous tient. COMMENT: les requetes DNS passent par des serveurs occidentaux — Google, Cloudflare — chaque visite est enregistree. VOILA: notre propre DNS = notre propre annuaire."),
            ] },
        EcoleLevel { slug: "m2", title: "Master 2 — Le Fondateur", cycle: "Sage",
            lessons: r#"<h3>Lecon 1: Creer une startup souveraine</h3><p>Pas de capital risque etranger. Financement: AFR tokens, revenus reels, cooperation africaine. Le modele: construire lentement, posseder 100%, ne jamais vendre. L'Afrique a assez de richesses pour financer ses propres Google, ses propres Amazon.</p><h3>Lecon 2: L'autonomie alimentaire</h3><p>L'Afrique importe 35 milliards de nourriture par an. Or 60% des terres arables du monde sont en Afrique. Le probleme: on exporte brut, on importe transforme. La solution: transformer sur place, vendre a valeur ajoutee, nourrir le continent d'abord.</p>"#,
            exercises: vec![
                ("Combien de nourriture l'Afrique importe par an?", "35 milliards", "POURQUOI: l'Afrique riche qui importe sa nourriture est une anomalie. COMMENT: 35 milliards de dollars de nourriture importee chaque an — alors que le continent possede 60% des terres arables du monde. VOILA: transformer sur place, nourrir le continent, exporter le surplus."),
                ("Quel pourcentage des terres arables du monde sont en Afrique?", "60", "POURQUOI: la securite alimentaire est une question de terre. COMMENT: 60% des terres arables non exploitees du monde sont en Afrique. Le probleme n'est pas la terre, c'est la transformation et l'acces au marche. VOILA: la solution est sous nos pieds."),
                ("Quelle monnaie AfriForme utilise pour le financement?", "afr", "POURQUOI: le financement etranger achete ta liberte. COMMENT: le capital-risque occidental prend des parts et dicte la direction. Le modele souverain: AFR tokens, revenus reels, cooperation africaine. VOILA: construire lentement, posseder 100%, ne jamais vendre."),
            ] },
        EcoleLevel { slug: "doctorat", title: "Doctorat — Le Sage", cycle: "Sage",
            lessons: r#"<h3>Lecon 1: L'heritage du Sage</h3><p>Le Sage ne garde pas le savoir — il le transmet. Tu as appris les empires, les langues, le code, l'economie. Maintenant tu enseignes. Le diplome du Sage n'est pas une fin — c'est un debut: tu retournes au village et tu transmets.</p><h3>Lecon 2: L'Afrique guide l'humanite</h3><p>En 2050: 2,5 milliards d'Africains, la moitie ont moins de 25 ans. L'Afrique n'est pas l'avenir de l'humanite — elle est l'humanite. Le baton revient a qui l'a fait naitre. Les Sages africains guideront le monde — avec sagesse, pas avec force.</p>"#,
            exercises: vec![
                ("Combien d'Africains en 2050?", "2,5 milliards", "POURQUOI: l'avenir demographique du monde est africain. COMMENT: en 2050, 2,5 milliards d'Africains — la moitie aura moins de 25 ans. La plus grande jeunesse du monde. VOILA: le continent qui aura les mains, les tetes, et la technologie."),
                ("Quel est le devoir du Sage?", "transmettre", "POURQUOI: le savoir garde est un savoir mort. COMMENT: le Sage a appris les empires, les langues, le code, l'economie — maintenant il retourne au village et enseigne. VOILA: la transmission est le diplome final."),
                ("Qui guidera l'humanite en 2050?", "les sages africains", "POURQUOI: le baton revient a qui l'a fait naitre. COMMENT: l'humanite est nee en Afrique — l'ADN de tout le monde vient d'ici. En 2050, la jeunesse africaine guidera avec sagesse, pas avec force. VOILA: l'Afrique guide l'humanite."),
            ] },
        EcoleLevel { slug: "s1", title: "Sciences 1 — La Matiere", cycle: "Science",
            lessons: r#"<h3>Lecon 1: L'atome et l'energie</h3><p>Toute matiere est faite d'atomes: un noyau (protons + neutrons) et des electrons qui tournent autour. L'energie solaire: un photon du soleil frappe un electron dans un panneau solaire, l'electron se libere et devient courant electrique. Voila comment le soleil africain devient electricite — pas de magie, de physique.</p><h3>Lecon 2: Cheikh Anta Diop, le physicien de l'histoire</h3><p>Cheikh Anta Diop (1923-1986): physicien et historien senegalais. Il a prouve par la physique nucleaire (datation au carbone 14) que l'Egypte ancienne etait noire africaine. Il a applique la science a l'histoire — le savant qui a rendu a l'Afrique sa memoire.</p>"#,
            exercises: vec![
                ("Quel savant a prouve l'Egypte ancienne africaine par la science?", "cheikh anta diop", "POURQUOI: l'histoire de l'Afrique devait etre prouvee par la science, pas par les recits du colon. COMMENT: Cheikh Anta Diop, physicien senegalais, a utilise la datation au carbone 14 et la melanine des momies pour prouver que les Egyptiens anciens etaient noirs africains. VOILA: la science rend a l'Afrique sa memoire."),
                ("Quelle particule du soleil frappe les panneaux solaires?", "photon", "POURQUOI: comprendre le solaire, c'est comprendre la matiere. COMMENT: le photon (grain de lumiere) frappe un electron dans le silicium du panneau, l'electron se libere et devient courant electrique. VOILA: chaque rayon africain est une particule d'electricite."),
                ("Quelle methode de datation Diop a-t-il utilisee?", "carbone 14", "POURQUOI: pour dater le passe, la physique offre un outil exact. COMMENT: le carbone 14 — un isotope radioactif qui decroisse avec le temps; en le mesurant on date un etre vivant mort il y a des milliers d'annees. VOILA: la methode qui a date les momies et restaure la verite."),
            ] },
        EcoleLevel { slug: "s2", title: "Sciences 2 — La Vie", cycle: "Science",
            lessons: r#"<h3>Lecon 1: La cellule et l'ADN</h3><p>Tout etre vivant est fait de cellules. Dans chaque cellule, l'ADN: la double helix qui porte le code de la vie. L'ADN humain est ne en Afrique — la premiere mere de l'humanite vivait ici. Ton ADN porte la memoire de millions d'annees de survie africaine.</p><h3>Lecon 2: La pharmacopee africaine</h3><p>L'artemisinine, le remede contre le paludisme, vient d'une plante africaine (Artemisia annua, feuille douce amere). Le kinkeliba, le neem, le moringa: des pharmacopees que les anciens connaissaient sans laboratoire. La science moderne confirme ce que les griots savaient.</p>"#,
            exercises: vec![
                ("Quelle molecule porte le code de la vie?", "adn", "POURQUOI: savoir ce que nous sommes commence dans la cellule. COMMENT: l'ADN — double helix dans chaque cellule, 3 milliards de lettres qui codent tout l'etre. VOILA: le livre de la vie, ecrit en 4 lettres, ne en Afrique."),
                ("Quel remede africain combat le paludisme?", "artemisinine", "POURQUOI: le paludisme tue des centaines de milliers d'Africains chaque annee. COMMENT: l'artemisinine, extraite d'Artemisia annua (la feuille douce-amere), detruit le parasite. Prix Nobel 2015 pour sa decouverte. VOILA: la plante africaine qui sauve le monde."),
                ("Sur quel continent est ne l'ADN humain?", "afrique", "POURQUOI: l'origine de l'humanite est une question scientifique resolue. COMMENT: la genetique comparee montre que toutes les lignees humaines remontent a une population africaine — Eve mitochondrielle vivait ici il y a 200 000 ans. VOILA: tout le monde est un cousin lointain de l'Afrique."),
            ] },
        EcoleLevel { slug: "s3", title: "Sciences 3 — Les Etoiles", cycle: "Science",
            lessons: r#"<h3>Lecon 1: Les Dogon et Sirius B</h3><p>Les Dogon du Mali connaissaient Sirius B — une etoile invisible a l'oeil nu — depuis des siecles, sans telescope. Les astronomes occidentaux ne l'ont confirmee qu'en 1862. Comment? En ecoutant le ciel, en comptant les cycles, en transmettant de generation en generation. L'astronomie africaine a precede le telescope.</p><h3>Lecon 2: Le cosmos et l'avenir</h3><p>L'univers a 13,8 milliards d'annees. Notre galaxie: 200 milliards d'etoiles. L'Afrique entre dans l'espace: satellites, observation, telecoms. Le ciel africain (desert du Sahara) est le meilleur du monde pour les telescopes — le desert devient observatoire.</p>"#,
            exercises: vec![
                ("Quel peuple connaissait Sirius B sans telescope?", "dogon", "POURQUOI: l'astronomie africaine precede les instruments occidentaux. COMMENT: les Dogon du Mali, par tradition orale, decrivaient Sirius B — invisible a l'oeil nu — son orbite de 50 ans, sa densite. VOILA: l'oreille et la memoire africaines ont vu ce que le telescope n'avait pas encore trouve."),
                ("Quelle etoile invisible les Dogon connaissaient?", "sirius b", "POURQUOI: Sirius B est le test de l'astronomie Dogon. COMMENT: Sirius B, le 'compagnon' de Sirius, est une naine blanche invisible a l'oeil nu — decouverte par les Occidentaux en 1862, connue des Dogon depuis des siecles. VOILA: la preuve que la tradition orale peut porter la science."),
                ("Combien d'annees a l'univers?", "13,8 milliards", "POURQUOI: situer l'humanite dans le cosmos donne l'echelle de nos reves. COMMENT: 13,8 milliards d'annees — mesure par l'expansion de l'univers et le rayonnement fossile. VOILA: notre galaxie a 200 milliards d'etoiles, et l'Afrique regarde maintenant vers elles."),
            ] },
        EcoleLevel { slug: "mat1", title: "Maths 1 — Les Nombres", cycle: "Maths",
            lessons: r#"<h3>Lecon 1: L'os d'Ishango, la premiere calculatrice</h3><p>L'os d'Ishango (Congo, 20000 ans): un os avec des encoches organisees en series mathematiques — premiers, paires. C'est la plus ancienne trace de calcul de l'humanite, trouvee en Afrique. L'humanite a appris a compter au Congo.</p><h3>Lecon 2: Les fractions egyptiennes</h3><p>Les Egyptiens utilisaient les fractions (1/2, 1/3, 1/4) pour partager le grain apres les crues du Nil. Le papyrus de Rhind (1650 av. J.-C.): un manuel de maths avec 87 problemes resolus. L'Afrique ecrivait des manuels de maths il y a 3600 ans.</p>"#,
            exercises: vec![
                ("Quel objet de 20000 ans est la premiere calculatrice?", "os d'ishango", "POURQUOI: les maths ont une histoire materielle. COMMENT: l'os d'Ishango, trouve au Congo, porte des encoches organisees en series mathematiques — nombres premiers, paires. VOILA: l'humanite a appris a compter en Afrique, il y a 20000 ans."),
                ("Dans quel pays a ete trouve l'os d'Ishango?", "congo", "POURQUOI: la premiere trace de calcul a une adresse. COMMENT: l'os d'Ishango vient des bords du lac Edouard, en Republique Democratique du Congo. VOILA: le Congo, berceau des mathematiques humaines."),
                ("Quel papyrus egyptien est un manuel de maths?", "rhind", "POURQUOI: les manuels d'Afrique precedent beaucoup d'ecritures. COMMENT: le papyrus de Rhind (1650 av. J.-C.) — 87 problemes resolus: fractions, equations, geometrie. VOILA: un manuel de maths africain vieux de 3600 ans."),
            ] },
        EcoleLevel { slug: "mat2", title: "Maths 2 — La Geometrie", cycle: "Maths",
            lessons: r#"<h3>Lecon 1: Les fractales africaines</h3><p>Les motifs du kente, les coiffures tressees, les villages circulaires: des fractales — des motifs qui se repetent a toutes les echelles. Le mathematicien Ron Eglash l'a prouve: l'Afrique utilisait la geometrie fractale des siecles avant que Mandelbrot ne la formalise. Nos ancetres calculaient en fractales sans le nommer.</p><h3>Lecon 2: Les pyramides et le nombre d'or</h3><p>La grande pyramide: 2,3 millions de blocs, alignee sur le vrai nord a 0,05 degre pres. Le nombre d'or (1,618) apparait dans ses proportions. Les geometres egyptiens connaissaient pi et le theoreme de Pythagore — 2000 ans avant Pythagore.</p>"#,
            exercises: vec![
                ("Quelle geometrie les motifs kente utilisent-ils?", "fractales", "POURQUOI: nos textiles portent des maths. COMMENT: les motifs du kente, les tresses, les villages circulaires sont des fractales — des motifs qui se repetent a toutes les echelles, formalisees par Mandelbrot au 20e siecle. VOILA: nos ancetres calculaient en fractales sans le nommer."),
                ("Quel mathematicien a prouve les fractales africaines?", "eglash", "POURQUOI: il fallait un regard scientifique sur nos motifs. COMMENT: Ron Eglash, mathematicien americain, a etudie les villages, textiles et coiffures d'Afrique et a prouve leur structure fractale dans 'African Fractals'. VOILA: la preuve scientifique de la geometrie africaine."),
                ("Quel nombre parfait apparait dans les pyramides?", "nombre d'or", "POURQUOI: les pyramides cachent des maths parfaites. COMMENT: le nombre d'or (1,618...) apparait dans les proportions de la grande pyramide; les geometres egyptiens connaissaient pi et Pythagore 2000 ans avant Pythagore. VOILA: la precision mathematique batie il y a 4500 ans."),
            ] },
        EcoleLevel { slug: "mat3", title: "Maths 3 — La Logique", cycle: "Maths",
            lessons: r#"<h3>Lecon 1: Les algorithmes</h3><p>Un algorithme: une suite d'etapes pour resoudre un probleme. Le mot vient d'Al-Khwarizmi, savant qui a formalise l'algebre. La division egyptienne, le tri d'un panier de mil, la route la plus courte au marche: des algorithmes africains avant le nom.</p><h3>Lecon 2: Les maths de la cryptographie</h3><p>Arithmetique modulaire: l'horloge tourne — apres 12 vient 1. Ed25519 utilise la courbe elliptique: des maths que les ordinateurs ne cassent pas. Le hash: entree → melange → sortie. Chaque transaction blockchain est un probleme de maths que seul le proprietaire peut signer.</p>"#,
            exercises: vec![
                ("De quel savant vient le mot algorithme?", "al-khwarizmi", "POURQUOI: meme les mots de l'informatique ont une histoire. COMMENT: le mot vient d'Al-Khwarizmi (780-850), savant de Bagdad ne en Perse, qui a formalise l'algebre — 'al-jabr'. VOILA: chaque fois que tu ecris un algorithme, tu prononces un heritage."),
                ("Quelle courbe Ed25519 utilise-t-il?", "elliptique", "POURQUOI: la securite d'AfriChain repose sur des maths solides. COMMENT: la courbe elliptique — des maths ou multiplier un point est facile mais retrouver le multiplicateur est impossible pour les ordinateurs. VOILA: une petite cle, une forteresse indechiffrable."),
                ("Comment s'appelle l'arithmetique de l'horloge?", "modulaire", "POURQUOI: la crypto moderne vit dans les nombres qui tournent. COMMENT: l'arithmetique modulaire — apres 12 vient 1, apres 255 vient 0. Les signatures et le minage calculent 'modulo' un grand nombre. VOILA: l'horloge est la clef des maths de la blockchain."),
            ] },
        EcoleLevel { slug: "tec1", title: "Techno 1 — L'Electricite", cycle: "Techno",
            lessons: r#"<h3>Lecon 1: Le circuit electrique</h3><p>Un circuit: source (batterie) → conducteur (fil) → charge (lampe) → retour. La tension (volts) pousse, le courant (amperes) coule, la resistance freine. L'Afrique a le soleil: un panneau solaire convertit la lumiere en courant continu, un regulateur protege la batterie, un onduleur transforme en courant alternatif. Voila une installation solaire complete.</p><h3>Lecon 2: La batterie et le stockage</h3><p>Le probleme du solaire: la nuit. Solution: batteries lithium ou... le sel! Des chercheurs africains testent le stockage par gravite (eau pompee le jour, turbine la nuit). L'energie stockee = souverainete energetique.</p>"#,
            exercises: vec![
                ("Quelle unite mesure la tension?", "volt", "POURQUOI: sans unite, pas d'installation solaire sure. COMMENT: le volt (V) mesure la pression electrique — comme la pression d'eau dans un tuyau. Un panneau solaire 12V alimente une batterie 12V. VOILA: la tension pousse, le courant coule."),
                ("Quelle unite mesure le courant?", "ampere", "POURQUOI: le courant determine la taille des fils et la securite. COMMENT: l'ampere (A) mesure le debit d'electrons — comme les litres par seconde d'un fleuve. Plus d'amperes = fils plus gros. VOILA: volts = pression, amperes = debit."),
                ("Quel composant protege la batterie solaire?", "regulateur", "POURQUOI: une batterie surchargee est une batterie morte. COMMENT: le regulateur de charge coupe ou reduit le courant quand la batterie est pleine, et empeche la decharge nocturne vers le panneau. VOILA: le gardien silencieux de l'installation."),
            ] },
        EcoleLevel { slug: "tec2", title: "Techno 2 — Les Reseaux", cycle: "Techno",
            lessons: r#"<h3>Lecon 1: Comment marche internet</h3><p>Une page web = une demande (requete HTTP) et une reponse. Le DNS traduit un nom (afriforme.com) en adresse IP. Le TCP decoupe les donnees en paquets, l'IP les route, le TCP les recolle. Internet n'est pas magique: c'est des lettres pliees en paquets.</p><h3>Lecon 2: Le mesh africain</h3><p>AfriMesh: pas de tour centrale. Chaque telephone est un noeud qui relaye les messages. UDP pour crier 'je suis la', TCP pour parler. Si un noeud tombe, les autres continuent — le reseau ne meurt jamais. C'est la difference avec le reseau occidental: eux ont un centre, nous avons un village.</p>"#,
            exercises: vec![
                ("Quel systeme traduit les noms en adresses IP?", "dns", "POURQUOI: l'annuaire d'internet est un pouvoir. COMMENT: le DNS (Domain Name System) traduit 'afriforme.com' en adresse IP numerique — comme l'annuaire du village traduit un nom en case. VOILA: qui controle le DNS vous controle — d'ou l'enjeu d'un DNS africain."),
                ("Quel protocole decoupe les donnees en paquets?", "tcp", "POURQUOI: envoyer un fichier d'un coup est fragile. COMMENT: le TCP decoupe les donnees en paquets numeros, l'IP les route a travers le monde, le TCP les recolle dans l'ordre et redemande les manquants. VOILA: internet est un courrier de paquets."),
                ("Quel reseau africain n'a pas de centre?", "afrimesh", "POURQUOI: un reseau a centre peut etre coupe. COMMENT: AfriMesh — chaque telephone est un noeud qui relaye les messages; si un noeud tombe, les autres continuent. UDP pour la decouverte, TCP pour les messages. VOILA: eux ont une tour, nous avons un village."),
            ] },
        EcoleLevel { slug: "tec3", title: "Techno 3 — L'Intelligence", cycle: "Techno",
            lessons: r#"<h3>Lecon 1: Comment apprend une machine</h3><p>Une IA apprend par exemples: on lui montre 1000 chats, elle extrait les motifs (oreilles, moustaches, forme). C'est l'apprentissage. Le Griot d'AfriForme: une base de connaissances africaines + des regles de reponse. Pas de serveur etranger — l'intelligence est dans la case.</p><h3>Lecon 2: L'IA souveraine</h3><p>Les IA occidentales sont entrainees sur les donnees du monde entier — dont les notres. Une IA africaine doit: apprendre en langues africaines, etre hebergee en Afrique, servir l'Afrique. Le projet: nos donnees, nos serveurs, notre intelligence. La machine qui pense en bambara vaut mille machines qui pensent a notre place.</p>"#,
            exercises: vec![
                ("Comment une IA apprend-elle?", "exemples", "POURQUOI: une machine nait vide, elle apprend comme un enfant. COMMENT: par exemples — on montre 1000 exemples, la machine extrait les motifs qui se repetent et les generalise. Plus d'exemples, plus de precision. VOILA: l'IA est une eleve qui n'oublie jamais."),
                ("Quelle IA hebergee en Afrique sert l'Afrique?", "ia souveraine", "POURQUOI: une IA etrangere sert d'abord son maitre. COMMENT: l'IA souveraine — donnees africaines, serveurs africains, langues africaines, decisions africaines. Comme Le Griot d'AfriForme. VOILA: notre intelligence dans notre case."),
                ("Dans quelle langue doit penser une IA africaine?", "bambara", "POURQUOI: penser dans sa langue, c'est penser sans traduction. COMMENT: une IA qui pense en bambara, wolof, swahili comprend le monde africain sans passer par l'anglais — nuances, proverbes, concepts uniques. VOILA: la machine qui pense en bambara vaut mille machines qui pensent a notre place."),
            ] },

    ]
}

fn ecole_normalize(s: &str) -> String {
    let mut out = String::new();
    for c in s.trim().to_lowercase().chars() {
        match c {
            'a'..='z' | '0'..='9' => out.push(c),
            'é' | 'è' | 'ê' | 'ë' => out.push('e'),
            'à' | 'â' | 'ä' => out.push('a'),
            'ô' | 'ö' => out.push('o'),
            'ù' | 'û' | 'ü' => out.push('u'),
            'î' | 'ï' => out.push('i'),
            'ç' => out.push('c'),
            ' ' | '-' | '\'' => out.push(' '),
            _ => {},
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}


fn html_honneur(state: &AppState) -> String {
    let cycles = ["Semence", "Griot", "Baobab", "Sage", "Science", "Maths", "Techno"];
    let mut body = String::new();
    body.push_str(r#"<div class="card" style="text-align:center;">
<h1>🏆 Le Tableau d'Honneur du Village</h1>
<p style="font-size:1.1em;">Les eleves qui brillent. Le village salue ceux qui etudient.</p>
<p style="color:#f59e0b;"><em>"Celui qui apprend eclaire le village entier."</em></p>
</div>"#);

    let mut eleves: Vec<(String, usize, Vec<String>)> = Vec::new();
    for (user, progress) in &state.ecole_progress {
        if progress.is_empty() { continue; }
        let mut dips: Vec<String> = Vec::new();
        for cyc in cycles {
            if ecole_cycle_complete(progress, cyc) { dips.push(cyc.to_string()); }
        }
        eleves.push((user.clone(), progress.len(), dips));
    }
    eleves.sort_by(|a, b| b.1.cmp(&a.1));

    if eleves.is_empty() {
        body.push_str("<div class='empty'>Personne n'a encore commence. Sois le premier — <a href='/ecole'>l'Ecole t'attend</a>.</div>");
    } else {
        let dip_names: std::collections::HashMap<&str, &str> = [
            ("Semence", "🌱 Semence"), ("Griot", "📖 Griot"), ("Baobab", "🌳 Baobab"),
            ("Sage", "🎓 Sage"), ("Science", "🔬 Savant"), ("Maths", "➗ Calculateur"), ("Techno", "⚙️ Ingenieur"),
        ].into_iter().collect();
        body.push_str("<div class='card'><h2>Les eleves du village</h2>");
        for (i, (user, count, dips)) in eleves.iter().enumerate() {
            let medal = if i == 0 { "🥇" } else if i == 1 { "🥈" } else if i == 2 { "🥉" } else { "⭐" };
            let dips_str = if dips.is_empty() {
                "<span style='color:#8b949e;'>aucun diplome encore</span>".to_string()
            } else {
                dips.iter().map(|d| format!("<a href='/diplome/{}' style='margin-right:6px;'>{}</a>", d, dip_names.get(d.as_str()).unwrap_or(&"🎓"))).collect::<Vec<_>>().join(" ")
            };
            body.push_str(&format!("<div class='repo' style='display:flex;justify-content:space-between;align-items:center;'><div>{} <strong>{}</strong><br><span style='font-size:0.9em;color:#8b949e;'>{} niveau(x) complete(s)</span></div><div>{}</div></div>", medal, user, count, dips_str));
        }
        body.push_str("</div>");
    }
    body.push_str("<div class='card'><a href='/ecole' class='btn'>🌱 Aller a l'Ecole</a></div>");
    html_page("Tableau d'Honneur", &body)
}

fn html_diplome_cert(user: &str, cycle: &str) -> String {
    let (title, emoji, desc) = match cycle {
        "Semence" => ("Diplome de la Semence", "🌱", "La graine est plantee, elle a germe. L'enfant connait sa terre, ses langues et ses empires."),
        "Griot" => ("Diplome du Griot", "📖", "Le jeune connait les histoires et peut les transmettre. Il garde la memoire du village."),
        "Baobab" => ("Diplome du Baobab", "🌳", "L'arbre de la sagesse. Il peut batir — code, economie, leadership — et guider les plus jeunes."),
        "Sage" => ("Diplome du Sage", "🎓", "Le Sage a appris et maintenant il enseigne. Il retourne au village et transmet la sagesse africaine."),
        "Science" => ("Diplome du Savant", "🔬", "Le Savant applique la science a l'Afrique: physique, biologie, astronomie. Comme Cheikh Anta Diop, il prouve par la science."),
        "Maths" => ("Diplome du Calculateur", "➗", "Le Calculateur herite d'Ishango: nombres, geometrie, logique. Les maths de l'Afrique, de l'os au blockchain."),
        _ => ("Diplome de l'Ingenieur", "⚙️", "L'Ingenieur construit: circuits solaires, reseaux mesh, IA souveraine. La technologie africaine entre ses mains."),
    };
    let today = afri_date_string();
    let body = format!(r#"<div style="text-align:center;">
<div style="border:3px double #f59e0b;border-radius:12px;padding:40px 20px;max-width:600px;margin:0 auto;background:#161b22;">
<p style="font-size:1.2em;color:#f59e0b;letter-spacing:3px;">ECOLE DU VILLAGE — AFRIFORME</p>
<h1 style="font-size:2.5em;margin:20px 0;">{} {}</h1>
<p style="font-size:1.3em;">decerne a</p>
<h2 style="color:#f59e0b;font-size:2em;margin:10px 0;">{}</h2>
<p style="font-size:1.1em;max-width:450px;margin:15px auto;">{}</p>
<p style="margin:25px 0;"><em>"On n'a vraiment appris que ce qu'on peut enseigner."</em></p>
<p style="color:#8b949e;">Fait au village, le {}</p>
<p style="color:#8b949e;font-size:0.9em;">Grave dans la memoire d'AfriForme — Rust std only, zero dependance</p>
</div>
<p style="margin:20px 0;">
<button onclick="window.print()" style="padding:10px 25px;font-size:1.1em;background:#238636;color:white;border:none;border-radius:8px;cursor:pointer;">🖨️ Imprimer le diplome</button>
</p>
<p><a href="/honneur" style="color:#58a6ff;">← Tableau d'Honneur</a></p>
</div>"#, emoji, title, user, desc, today);
    html_page("Diplome", &body)
}

fn ecole_cycle_complete(progress: &[String], cycle: &str) -> bool {
    let levels = ecole_levels();
    levels.iter().all(|l| {
        l.cycle != cycle || progress.iter().any(|p| p == l.slug)
    })
}

fn html_explore(state: &AppState) -> String {
    let public_repos: Vec<&Repository> = state.repos.iter().filter(|r| r.is_public).collect();
    let repos_html = if public_repos.is_empty() {
        "<div class='empty'>Aucun depot public pour le moment.</div>".to_string()
    } else {
        public_repos.iter().map(|r| format!(
            r#"<div class="repo"><h3><a href="/{}/{}">{}/{}</a> <span class="badge badge-{}">{}</span></h3><div class="desc">{}</div><div class="meta">⭐ {} · 🍴 {} · {}</div></div>"#,
            r.owner, r.name, r.owner, r.name,
            if r.language == "Rust" { "rust" } else if r.language == "Python" { "python" } else { "js" },
            r.language, r.description, r.stars, r.forks, ts_lisible(&r.created_at)
        )).collect::<Vec<_>>().join("")
    };

    // Trending: top 5 by (stars * 2 + forks)
    let mut trending: Vec<&Repository> = public_repos.clone();
    trending.sort_by(|a, b| {
        let score_a = a.stars * 2 + a.forks;
        let score_b = b.stars * 2 + b.forks;
        score_b.cmp(&score_a)
    });
    let trending_html = if trending.is_empty() {
        "<div class='empty'>Aucun depot en tendance.</div>".to_string()
    } else {
        trending.iter().take(5).enumerate().map(|(i, r)| {
            let score = r.stars * 2 + r.forks;
            let rank: String = if i == 0 { "🔥".to_string() } else { format!("{}.", i + 1) };
            format!(
                r#"<div class="repo" style="border-left:3px solid #f59e0b;"><h3>{} <a href="/{}/{}">{}/{}</a> <span class="badge badge-{}">{}</span></h3><div class="desc">{}</div><div class="meta">⭐ {} · 🍴 {} · Score: {}</div></div>"#,
                rank,
                r.owner, r.name, r.owner, r.name,
                if r.language == "Rust" { "rust" } else if r.language == "Python" { "python" } else { "js" },
                r.language, r.description, r.stars, r.forks, score
            )
        }).collect::<Vec<_>>().join("")
    };

    let body = format!(r#"
<h1>🦁 La Savane — les depots</h1>
<p style="color:#8b949e;">Decouvrez les projets de la communaute africaine</p>
<h2>🔥 La Braise — depots brulants</h2>
{}
<h2>📦 Tous les depots</h2>
{}
"#, trending_html, repos_html);

    html_page("La Savane", &body)
}

fn html_ai_chat(username: &str, chat: Option<&AIChat>) -> String {
    let messages_html = if let Some(c) = chat {
        c.messages.iter().map(|(role, content)| {
            let (label, cls) = if role == "user" { ("Toi", "user") } else { ("🤖 IA", "ai") };
            format!(r#"<div class="ai-msg {}"><div class="role">{}</div><div class="content">{}</div></div>"#, cls, label, content)
        }).collect::<Vec<_>>().join("")
    } else {
        "<div class='empty'>Pose ta premiere question a l'IA!</div>".to_string()
    };

    let body = format!(r#"
<h1>📖 Le Griot — IA Africaine</h1>
<p style="color:#8b949e;">Ton assistant de code — demande-moi tout sur Rust, Python, blockchain, Termux, Git, AfriChain</p>
<div class="ai-chat">
{}
</div>
<form method="POST" action="/ai">
<input type="text" name="question" placeholder="Pose ta question..." required>
<button type="submit">Demander</button>
</form>
<div style="margin-top:15px;">
<h3>Suggestions:</h3>
<p><a href="/ai?q=comment+creer+une+fonction+en+rust">Comment creer une fonction en Rust?</a></p>
<p><a href="/ai?q=comment+utiliser+termux">Comment utiliser Termux?</a></p>
<p><a href="/ai?q=qu+est+ce+que+la+blockchain">Qu'est-ce que la blockchain?</a></p>
<p><a href="/ai?q=parle+moi+de+africhain">Parle-moi d'AfriChain</a></p>
</div>
"#, messages_html);

    html_page("Le Griot", &body)
}

// ============================================================
// COURSE HTML PAGES
// ============================================================

fn html_course(course: &Course, repo: &Repository, owner: &str, name: &str, current_user: Option<&str>) -> String {
    // Check progress
    let completed: Vec<usize> = if let Some(user) = current_user {
        course.progress.get(user)
            .map(|s| s.split(',').filter_map(|n| n.parse::<usize>().ok()).collect())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let total_ex = course.exercises.len();
    let completed_count = completed.len();
    let progress_pct = if total_ex > 0 { (completed_count * 100) / total_ex } else { 0 };
    let all_done = completed_count == total_ex && total_ex > 0;

    // Modules HTML
    let modules_html = course.modules.iter().enumerate().map(|(i, m)| {
        format!(
            r#"<div class="card"><h3>📖 Module {}: {}</h3><div style="white-space:pre-wrap;color:#c9d1d9;">{}</div></div>"#,
            i + 1, m.title, m.content
        )
    }).collect::<Vec<_>>().join("");

    // Exercises HTML
    let exercises_html = course.exercises.iter().enumerate().map(|(i, ex)| {
        let is_done = completed.contains(&i);
        let status_badge = if is_done {
            "<span class='badge' style='background:#2ea043;color:#fff;'>Reussi</span>"
        } else {
            "<span class='badge' style='background:#30363d;color:#8b949e;'>A faire</span>"
        };

        let form_html = if let Some(_user) = current_user {
            if is_done {
                // NE PAS montrer la reponse — le prof evalue en secret
                r#"<div style="margin-top:10px;padding:10px;background:#0d1117;border-radius:6px;border:1px solid #2ea043;">
                <div style="color:#2ea043;">✅ Reussi! Le prof a valide ta reponse.</div>
                </div>"#.to_string()
            } else {
                format!(
                    r#"<form method="POST" action="/course/{}/{}/answer" style="margin-top:10px;">
                    <input type="hidden" name="ex_index" value="{}">
                    <input type="text" name="answer" placeholder="Ta reponse..." required>
                    <button type="submit">Valider</button>
                    </form>"#,
                    owner, name, i
                )
            }
        } else {
            "<div style='color:#8b949e;margin-top:5px;font-size:0.85em;'>Connecte-toi pour repondre aux exercices.</div>".to_string()
        };

        format!(
            r#"<div class="card"><h3>✏️ Exercice {} {}</h3><p style="color:#c9d1d9;">{}</p>{}</div>"#,
            i + 1, status_badge, ex.question, form_html
        )
    }).collect::<Vec<_>>().join("");

    // Diploma section
    let diploma_html = if all_done {
        format!(
            r#"<div class="card" style="border:2px solid #f59e0b;background:#1a1500;text-align:center;padding:30px;">
            <div style="font-size:3em;">🎓</div>
            <h2 style="color:#f59e0b;">FELICITATIONS!</h2>
            <p style="font-size:1.2em;color:#c9d1d9;">Tu as complete tous les exercices du cours!</p>
            <div style="margin:20px 0;padding:20px;border:2px dashed #f59e0b;border-radius:8px;">
            <h3 style="color:#f59e0b;">DIPLOME: {}</h3>
            <p style="color:#8b949e;">Decerne a: {}</p>
            <p style="color:#8b949e;font-size:0.8em;">Projet: {}/{} | Cours: {} | Exercices: {}/{}</p>
            </div>
            <p style="color:#2ea043;">Ce diplome est grave dans la memoire d'AfriForme. L'Afrique construit, l'Afrique apprend, l'Afrique enseigne.</p>
            </div>"#,
            course.diploma_name,
            current_user.unwrap_or("Anonyme"),
            owner, name, course.title, completed_count, total_ex
        )
    } else if completed_count > 0 {
        format!(
            r#"<div class="card" style="border:1px solid #30363d;text-align:center;padding:20px;">
            <div style="font-size:2em;">🎓</div>
            <h3>Diplome: {}</h3>
            <p style="color:#8b949e;">Progression: {}/{} exercices ({}%)</p>
            <div style="background:#21262d;border-radius:6px;height:20px;margin:10px 0;overflow:hidden;">
            <div style="background:#f59e0b;height:100%;width:{}%;">{}</div>
            </div>
            <p style="color:#8b949e;font-size:0.85em;">Complete tous les exercices pour obtenir ton diplome!</p>
            </div>"#,
            course.diploma_name, completed_count, total_ex, progress_pct, progress_pct,
            if progress_pct > 0 { "&nbsp;" } else { "" }
        )
    } else {
        format!(
            r#"<div class="card" style="border:1px solid #30363d;text-align:center;padding:20px;">
            <div style="font-size:2em;">🎓</div>
            <h3>Diplome: {}</h3>
            <p style="color:#8b949e;">Complete tous les exercices pour obtenir ton diplome!</p>
            </div>"#,
            course.diploma_name
        )
    };

    let body = format!(r#"
<div style="display:flex;justify-content:space-between;align-items:center;">
<div>
<h1>🎓 {}</h1>
<p style="color:#8b949e;">{}</p>
<p><a href="/{}/{}">← Retour au depot</a></p>
</div>
<div class="stats">
<div class="stat"><div class="num">{}</div><div class="label">Modules</div></div>
<div class="stat"><div class="num">{}/{}</div><div class="label">Exercices</div></div>
<div class="stat"><div class="num">{}%</div><div class="label">Progression</div></div>
</div>
</div>
<h2>📚 Modules du Cours</h2>
{}
<h2>✏️ Exercices</h2>
{}
{}
"#, course.title, course.description, owner, name,
course.modules.len(), completed_count, total_ex, progress_pct,
modules_html, exercises_html, diploma_html);

    html_page(&format!("Cours: {}", course.title), &body)
}



fn html_notifications(state: &AppState, current_user: Option<&str>) -> String {
    let user_section = if let Some(user) = current_user {
        let user_notifs: Vec<&Notification> = state.get_user_notifications(user);
        let unread = state.get_unread_count(user);
        let notif_html = if user_notifs.is_empty() {
            "<div class='empty'>Aucune notification. Tu es a jour!</div>".to_string()
        } else {
            user_notifs.iter().rev().map(|n| {
                let icon = if n.read { "📭" } else { "🔔" };
                let style = if n.read { "opacity:0.6;" } else { "border-left:3px solid #f59e0b;" };
                format!(
                    r#"<div class="card" style="{}"><span style="font-size:1.2em;">{}</span> <span style="color:#8b949e;font-size:0.8em;">· {}</span><br>{} <a href="{}" style="color:#58a6ff;">Voir</a></div>"#,
                    style, icon, n.created_at, n.message, n.link
                )
            }).collect::<Vec<_>>().join("")
        };

        format!(r#"
<h1>🥁 Le Tambour</h1>
<div class="stats">
<div class="stat"><div class="num">{}</div><div class="label">Non lues</div></div>
<div class="stat"><div class="num">{}</div><div class="label">Total</div></div>
</div>
{}
"#, unread, user_notifs.len(), notif_html)
    } else {
        format!(r#"
<h1>🥁 Le Tambour</h1>
<div class='empty'>Connecte-toi pour voir tes notifications.</div>
<a href="/login" class="btn">Connexion</a>
"#)
    };

    html_page("Tambour", &user_section)
}

// ============================================================
// AFRI-ZIP — Constructeur ZIP pur (zero dependance)
// Format ZIP stocke (sans compression), CRC32 maison
// ============================================================

fn crc32_afri(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB88320 } else { crc >> 1 };
        }
    }
    !crc
}

fn push_u16(v: &mut Vec<u8>, val: u16) {
    v.push((val & 0xFF) as u8);
    v.push((val >> 8) as u8);
}

fn push_u32(v: &mut Vec<u8>, val: u32) {
    v.push((val & 0xFF) as u8);
    v.push(((val >> 8) & 0xFF) as u8);
    v.push(((val >> 16) & 0xFF) as u8);
    v.push(((val >> 24) & 0xFF) as u8);
}

fn build_zip(files: &[(String, String)]) -> Vec<u8> {
    let mut zip: Vec<u8> = Vec::new();
    let mut central: Vec<u8> = Vec::new();
    let mut count: u16 = 0;

    for (name, content) in files {
        let data = content.as_bytes();
        let crc = crc32_afri(data);
        let size = data.len() as u32;
        let name_bytes = name.as_bytes();
        let offset = zip.len() as u32;

        // En-tete local: PK\x03\x04
        zip.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
        push_u16(&mut zip, 20);      // version minimale
        push_u16(&mut zip, 0);       // flags
        push_u16(&mut zip, 0);       // methode 0 = stocke
        push_u16(&mut zip, 0);       // heure
        push_u16(&mut zip, 0x21);    // date (1980-01-01 approx)
        push_u32(&mut zip, crc);
        push_u32(&mut zip, size);    // taille compressee
        push_u32(&mut zip, size);    // taille originale
        push_u16(&mut zip, name_bytes.len() as u16);
        push_u16(&mut zip, 0);       // extra len
        zip.extend_from_slice(name_bytes);
        zip.extend_from_slice(data);

        // Enregistrement central: PK\x01\x02
        central.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
        push_u16(&mut central, 20);  // version fait par
        push_u16(&mut central, 20);  // version minimale
        push_u16(&mut central, 0);   // flags
        push_u16(&mut central, 0);   // methode
        push_u16(&mut central, 0);   // heure
        push_u16(&mut central, 0x21);// date
        push_u32(&mut central, crc);
        push_u32(&mut central, size);
        push_u32(&mut central, size);
        push_u16(&mut central, name_bytes.len() as u16);
        push_u16(&mut central, 0);   // extra
        push_u16(&mut central, 0);   // commentaire
        push_u16(&mut central, 0);   // disque
        push_u16(&mut central, 0);   // interne
        push_u32(&mut central, 0);   // externe
        push_u32(&mut central, offset);
        central.extend_from_slice(name_bytes);

        count += 1;
    }

    let central_offset = zip.len() as u32;
    zip.extend_from_slice(&central);

    // Fin de central: PK\x05\x06
    zip.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
    push_u16(&mut zip, 0);
    push_u16(&mut zip, 0);
    push_u16(&mut zip, count);       // entrees sur ce disque
    push_u16(&mut zip, count);       // entrees total
    push_u32(&mut zip, central.len() as u32);
    push_u32(&mut zip, central_offset);
    push_u16(&mut zip, 0);           // commentaire len

    zip
}

// ============================================================
// Jeux du Village — Le Lion du Sahel (v0.18)
// Chaque piece ramassee dans le jeu va sur le compte du joueur.
// ============================================================
const GAME_HTML: &str = r##"<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1,user-scalable=no">
<title>Le Lion du Sahel — La Vraie Savane</title>
<style>
body{margin:0;background:#000;overflow:hidden;touch-action:manipulation;user-select:none;-webkit-user-select:none}
canvas{display:block}
#hud{position:fixed;top:10px;left:14px;right:14px;display:flex;justify-content:space-between;align-items:flex-start;color:#fff;font-family:Georgia,serif;text-shadow:0 2px 6px rgba(0,0,0,.7);z-index:2;pointer-events:none}
#score{font-size:24px;font-weight:bold;letter-spacing:1px}
#best{font-size:12px;opacity:.85;margin-top:2px}
#phase{font-size:12px;margin-top:4px;opacity:.9}
#coins{font-size:18px;font-weight:bold}
#srv{position:fixed;top:64px;right:14px;color:#ffd54a;font-family:Georgia,serif;font-size:11px;text-shadow:0 2px 4px #000;z-index:2;pointer-events:none;text-align:right}
#msg{position:fixed;bottom:16px;width:100%;text-align:center;color:#fff;font-family:Georgia,serif;font-size:14px;text-shadow:0 2px 6px #000;z-index:2;pointer-events:none;opacity:.9}
</style></head><body>
<canvas id="cv"></canvas>
<div id="hud">
<div><div id="score">0</div><div id="best">Record: <span id="hiscore">0</span></div><div id="phase"></div></div>
<div id="coins">🪙 0</div>
</div>
<div id="srv"></div>
<div id="msg">🦁 Touche l'écran pour sauter — double saut permis !</div>
<script>
const cv=document.getElementById('cv'),ctx=cv.getContext('2d');
let W,H,hor;
function fit(){W=cv.width=innerWidth;H=cv.height=innerHeight;hor=H*0.72;}
fit();addEventListener('resize',fit);

// ---------- couleurs du ciel : cycle jour/nuit reel ----------
const SKY=[
 [255,150,58, 255,190,100, 255,228,170, 0.00,'🌅 Heure dorée'],
 [120,48,95, 225,95,60, 255,175,80, 0.25,'🌇 Coucher du soleil'],
 [40,30,70, 90,55,95, 210,105,80, 0.50,'🌆 Crépuscule'],
 [8,8,24, 18,22,55, 30,32,75, 0.75,'🌌 Nuit du Sahel'],
 [70,80,125, 225,120,140, 255,215,165, 0.25,'🌄 Aube'],
 [255,150,58, 255,190,100, 255,228,170, 0.00,'🌅 Heure dorée']
];
function lerp(a,b,t){return a+(b-a)*t;}
function lerpC(a,b,t){return [lerp(a[0],b[0],t),lerp(a[1],b[1],t),lerp(a[2],b[2],t)];}
function rgb(c){return 'rgb('+(c[0]|0)+','+(c[1]|0)+','+(c[2]|0)+')';}
function skyAt(t){
  const n=SKY.length-1, f=t*n, i=Math.min(n-1,f|0), u=f-i;
  const A=SKY[i],B=SKY[i+1];
  return {top:lerpC(A,B,u), mid:lerpC(A.slice(3,6),B.slice(3,6),u), bot:lerpC(A.slice(6,9),B.slice(6,9),u),
          night:lerp(A[9],B[9],u), name:(u<0.5?A[10]:B[10])};
}

// ---------- decor genere ----------
let stars=[],clouds=[],trees=[],tufts=[],mtn1=[],mtn2=[],dots=[];
function seedRand(s){return function(){s=(s*16807)%2147483647;return (s&0xffff)/0xffff;};}
function genWorld(){
  const r=seedRand(1234567);
  stars=[];for(let i=0;i<130;i++)stars.push({x:r()*1.2,y:r()*0.55,p:r()*6.28,w:1+r()*2});
  clouds=[];for(let i=0;i<5;i++)clouds.push({x:r(),y:0.08+r()*0.22,s:0.5+r(),v:0.004+r()*0.008});
  trees=[];for(let i=0;i<7;i++)trees.push({x:0.1+i*0.15+r()*0.05,s:0.7+r()*0.9});
  tufts=[];for(let i=0;i<26;i++)tufts.push({x:r(),s:0.5+r()});
  mtn1=[];for(let i=0;i<=24;i++)mtn1.push(0.10+r()*0.13);
  mtn2=[];for(let i=0;i<=18;i++)mtn2.push(0.05+r()*0.09);
  dots=[];for(let i=0;i<50;i++)dots.push({x:r(),y:r(),s:0.5+r()});
}
genWorld();

// ---------- etat du jeu ----------
let S='menu',score=0,coins=0,speed=0,scroll=0,dayT=0,timeMs=0;
let lion={y:0,vy:0,jumps:0,ph:0};
let obs=[],cns=[],puffs=[];
const G=2600,JUMP=950,GR=0.72;
let hi=+(localStorage.getItem('lionHi')||0);
document.getElementById('hiscore').textContent=hi;
fetch('/jeux/score').then(r=>r.json()).then(d=>{if(d.ok)document.getElementById('srv').innerHTML='🪙 Compte: '+d.coins+' pièces · Record: '+d.best;}).catch(()=>{});

function reset(){
  score=0;coins=0;speed=340;scroll=0;obs=[];cns=[];puffs=[];
  lion.y=0;lion.vy=0;lion.jumps=0;lion.ph=0;
  document.getElementById('msg').textContent='';
}
function start(){reset();S='play';}

// ---------- entrees ----------
function tap(){
  if(S==='menu'){start();return;}
  if(S==='over'){if(timeMs-overAt>600)start();return;}
  if(lion.jumps<2){lion.vy=-JUMP*(lion.jumps===0?1:0.88);lion.jumps++;
    for(let i=0;i<6;i++)puffs.push({x:W*0.18,y:hor-lion.y,vx:-60-Math.random()*80,vy:20+Math.random()*50,l:0.5,r:3+Math.random()*4});}
}
addEventListener('pointerdown',tap);
addEventListener('keydown',e=>{if(e.code==='Space')tap();});

// ---------- spawn ----------
let nextObs=600,nextCns=900;
function spawn(dt){
  scroll+=speed*dt;
  nextObs-=speed*dt;nextCns-=speed*dt;
  if(nextObs<=0){
    nextObs=420+Math.random()*520*(340/speed+0.4);
    const k=Math.random();
    if(k<0.34)obs.push({t:'cactus',x:W+60,w:26,h:58});
    else if(k<0.62)obs.push({t:'rocher',x:W+60,w:44,h:34});
    else if(k<0.86)obs.push({t:'termitiere',x:W+60,w:40,h:64});
    else obs.push({t:'aigle',x:W+60,w:64,h:30,fly:60+Math.random()*90,ph:Math.random()*6});
  }
  if(nextCns<=0){
    nextCns=700+Math.random()*900;
    const n=3+(Math.random()*3|0),bx=W+60,arc=Math.random()<0.5;
    for(let i=0;i<n;i++)cns.push({x:bx+i*46,y:arc?Math.sin(i/(n-1)*Math.PI)*120:0,ph:Math.random()*6});
  }
}

// ---------- dessin : decor ----------
function drawSky(sky){
  const g=ctx.createLinearGradient(0,0,0,hor);
  g.addColorStop(0,rgb(sky.top));g.addColorStop(0.55,rgb(sky.mid));g.addColorStop(1,rgb(sky.bot));
  ctx.fillStyle=g;ctx.fillRect(0,0,W,hor+2);
  // etoiles
  if(sky.night>0.08){
    for(const st of stars){
      const a=sky.night*(0.35+0.65*Math.abs(Math.sin(timeMs*0.001+st.p)));
      ctx.fillStyle='rgba(255,250,220,'+a.toFixed(2)+')';
      ctx.fillRect(st.x*W,st.y*H,st.w,st.w);
    }
  }
  // soleil
  const sunAlt=1-Math.min(1,dayT/0.32);
  if(sunAlt>0){
    const sx=W*0.76,sy=hor-sunAlt*H*0.5-10;
    const sg=ctx.createRadialGradient(sx,sy,0,sx,sy,90);
    sg.addColorStop(0,'rgba(255,240,180,0.95)');sg.addColorStop(0.25,'rgba(255,190,90,0.55)');sg.addColorStop(1,'rgba(255,150,50,0)');
    ctx.fillStyle=sg;ctx.beginPath();ctx.arc(sx,sy,90,0,7);ctx.fill();
    ctx.fillStyle='rgb(255,235,170)';ctx.beginPath();ctx.arc(sx,sy,26,0,7);ctx.fill();
  }
  // lune
  const mT=(dayT-0.30)/0.48;
  if(mT>0&&mT<1){
    const mx=W*0.22,my=hor-Math.sin(mT*Math.PI)*H*0.5-10;
    ctx.fillStyle='rgba(235,235,220,0.95)';ctx.beginPath();ctx.arc(mx,my,20,0,7);ctx.fill();
    ctx.fillStyle=rgb(sky.top);ctx.beginPath();ctx.arc(mx-9,my-5,17,0,7);ctx.fill();
  }
  // nuages
  for(const c of clouds){
    c.x-=c.v*0.016*(1+speed/600);if(c.x<-0.25)c.x=1.25;
    const cy=c.y*H,cw=70*c.s;
    const col=lerpC([255,230,200],[40,40,70],sky.night);
    ctx.fillStyle='rgba('+(col[0]|0)+','+(col[1]|0)+','+(col[2]|0)+',0.5)';
    ctx.beginPath();
    ctx.ellipse(c.x*W,cy,cw,14*c.s,0,0,7);
    ctx.ellipse(c.x*W-cw*0.5,cy+6,cw*0.55,10*c.s,0,0,7);
    ctx.ellipse(c.x*W+cw*0.55,cy+7,cw*0.5,9*c.s,0,0,7);
    ctx.fill();
  }
}
function drawRidge(arr,par,col,base){
  const n=arr.length-1,seg=W/n;
  ctx.fillStyle=col;ctx.beginPath();ctx.moveTo(0,hor+2);
  for(let i=0;i<=n;i++){
    const x=(i*seg-(scroll*par)%(W))+(scroll*par>W?-W:0);
    ctx.lineTo(x,hor-arr[i]*H*2.4-base);
  }
  ctx.lineTo(W,hor+2);ctx.closePath();ctx.fill();
}
function acacia(x,y,s,sky){
  const dark=lerpC([70,45,25],[12,10,20],sky.night);
  ctx.fillStyle=rgb(dark);
  ctx.fillRect(x-3*s,y-46*s,6*s,46*s);
  ctx.beginPath();
  ctx.moveTo(x-2*s,y-40*s);ctx.lineTo(x-26*s,y-52*s);ctx.lineTo(x-24*s,y-55*s);ctx.lineTo(x-2*s,y-46*s);ctx.closePath();ctx.fill();
  ctx.beginPath();
  ctx.moveTo(x+2*s,y-42*s);ctx.lineTo(x+24*s,y-54*s);ctx.lineTo(x+22*s,y-57*s);ctx.lineTo(x+2*s,y-48*s);ctx.closePath();ctx.fill();
  ctx.beginPath();ctx.ellipse(x,y-58*s,34*s,10*s,0,0,7);ctx.fill();
  ctx.beginPath();ctx.ellipse(x-8*s,y-66*s,20*s,7*s,0,0,7);ctx.fill();
}
function drawGround(sky){
  const g=ctx.createLinearGradient(0,hor,0,H);
  const c1=lerpC([150,105,55],[28,22,38],sky.night),c2=lerpC([105,72,38],[16,13,26],sky.night);
  g.addColorStop(0,rgb(c1));g.addColorStop(1,rgb(c2));
  ctx.fillStyle=g;ctx.fillRect(0,hor,W,H-hor);
  // cailloux / texture
  ctx.fillStyle='rgba(0,0,0,0.18)';
  for(const d of dots){
    const x=((d.x*W*2-scroll)%(W*2)+W*2)%(W*2)-W*0.5;
    ctx.beginPath();ctx.ellipse(x,hor+8+d.y*(H-hor-14),4*d.s,2*d.s,0,0,7);ctx.fill();
  }
  // herbes premier plan
  const gc=lerpC([120,140,50],[20,30,25],sky.night);
  ctx.strokeStyle=rgb(gc);ctx.lineWidth=2;
  for(const t of tufts){
    const x=((t.x*W*2-scroll*1.15)%(W*2)+W*2)%(W*2)-W*0.5;
    const y=hor+14+t.s*(H-hor-20),s=t.s*14,sw=Math.sin(timeMs*0.003+x)*3;
    ctx.beginPath();ctx.moveTo(x,y);ctx.quadraticCurveTo(x+sw,y-s*0.6,x+sw*2,y-s);
    ctx.moveTo(x+5,y);ctx.quadraticCurveTo(x+5+sw,y-s*0.5,x+6+sw*2,y-s*0.85);
    ctx.stroke();
  }
}

// ---------- dessin : lion ----------
function drawLion(sky){
  const x=W*0.18,y=hor-lion.y,bob=Math.sin(lion.ph*2)*2.5;
  const run=S==='play'&&lion.y<=0.5;
  ctx.save();ctx.translate(x,y+bob);
  const dark=lerpC([0,0,0],[0,0,0],0);
  // queue
  ctx.strokeStyle='#8a5a28';ctx.lineWidth=5;ctx.lineCap='round';
  ctx.beginPath();ctx.moveTo(-34,-26);
  ctx.quadraticCurveTo(-58,-34+Math.sin(timeMs*0.01)*6,-60,-52+Math.sin(timeMs*0.01)*8);
  ctx.stroke();
  ctx.fillStyle='#6a3a18';ctx.beginPath();ctx.arc(-60,-54+Math.sin(timeMs*0.01)*8,7,0,7);ctx.fill();
  // pattes
  const lp=run?Math.sin(lion.ph*2):0;
  ctx.fillStyle='#b57a30';
  for(const [ox,ph] of [[-22,0],[-14,Math.PI],[16,Math.PI],[24,0]]){
    const sw=run?Math.sin(lion.ph*2+ph)*10:0;
    ctx.save();ctx.translate(ox+sw*0.4,-14);
    ctx.rotate(run?Math.sin(lion.ph*2+ph)*0.35:0);
    ctx.fillRect(-4,0,8,16);ctx.restore();
  }
  // corps
  ctx.fillStyle='#c8862e';
  ctx.beginPath();ctx.ellipse(0,-26,36,17,0,0,7);ctx.fill();
  ctx.fillStyle='#e0a856';
  ctx.beginPath();ctx.ellipse(2,-18,26,10,0,0,7);ctx.fill();
  // criniere
  ctx.fillStyle='#7a4218';
  ctx.beginPath();ctx.arc(34,-34,17,0,7);ctx.fill();
  for(let i=0;i<8;i++){const a=i/8*6.28;
    ctx.beginPath();ctx.arc(34+Math.cos(a)*17,-34+Math.sin(a)*17,5,0,7);ctx.fill();}
  // tete
  ctx.fillStyle='#d09040';
  ctx.beginPath();ctx.arc(34,-34,12,0,7);ctx.fill();
  // oreille
  ctx.beginPath();ctx.arc(28,-45,5,0,7);ctx.fill();
  // museau
  ctx.beginPath();ctx.ellipse(43,-30,7,5,0,0,7);ctx.fill();
  ctx.fillStyle='#5a3010';ctx.beginPath();ctx.arc(46,-30,2.5,0,7);ctx.fill();
  // oeil
  ctx.fillStyle='#1a0f05';ctx.beginPath();ctx.arc(37,-37,2.4,0,7);ctx.fill();
  if(sky.night>0.5){ctx.fillStyle='rgba(255,220,120,0.9)';ctx.beginPath();ctx.arc(37,-37,1.2,0,7);ctx.fill();}
  ctx.restore();
}

// ---------- dessin : obstacles & pieces ----------
function drawObs(sky){
  const shade=lerpC([0,0,0],[0,0,0],0);
  for(const o of obs){
    const x=o.x,y=o.t==='aigle'?hor-o.fly-o.h+Math.sin(timeMs*0.004+o.ph)*14:hor-o.h;
    if(o.t==='cactus'){
      ctx.fillStyle=lerpC([46,110,58],[14,30,26],sky.night)?rgb(lerpC([46,110,58],[14,30,26],sky.night)):'#2e6e3a';
      ctx.fillRect(x-8,y,16,o.h);
      ctx.fillRect(x-22,y+o.h*0.35,14,8);ctx.fillRect(x-22,y+o.h*0.35,8,o.h*0.3);
      ctx.fillRect(x+8,y+o.h*0.2,14,8);ctx.fillRect(x+14,y+o.h*0.2,8,o.h*0.35);
      ctx.fillStyle='rgba(255,255,255,0.25)';ctx.fillRect(x-8,y,4,o.h);
    }else if(o.t==='rocher'){
      const g=ctx.createLinearGradient(x,y,x,y+o.h);
      g.addColorStop(0,'#8a8a92');g.addColorStop(1,'#4a4a52');
      ctx.fillStyle=g;ctx.beginPath();ctx.ellipse(x,y+o.h*0.6,o.w/2,o.h*0.62,0,Math.PI,0);ctx.fill();
      ctx.fillStyle='rgba(255,255,255,0.18)';ctx.beginPath();ctx.ellipse(x-6,y+o.h*0.35,8,5,-0.5,0,7);ctx.fill();
    }else if(o.t==='termitiere'){
      ctx.fillStyle='#9a5a2c';
      ctx.beginPath();ctx.moveTo(x-o.w/2,y+o.h);ctx.quadraticCurveTo(x-o.w/2,y+o.h*0.3,x,y);
      ctx.quadraticCurveTo(x+o.w/2,y+o.h*0.3,x+o.w/2,y+o.h);ctx.closePath();ctx.fill();
      ctx.fillStyle='#6a3a18';
      for(let i=0;i<4;i++)ctx.fillRect(x-8+((i*13)%16),y+o.h*0.25+i*o.h*0.16,3,3);
    }else{ // aigle
      const fl=Math.sin(timeMs*0.012+o.ph);
      ctx.fillStyle='#2a2018';
      ctx.beginPath();ctx.ellipse(x+10,y+o.h/2,16,7,0,0,7);ctx.fill();
      ctx.beginPath();ctx.moveTo(x+2,y+o.h/2);ctx.lineTo(x-14,y+o.h/2-16*fl);ctx.lineTo(x+6,y+o.h/2+2);ctx.closePath();ctx.fill();
      ctx.beginPath();ctx.moveTo(x+16,y+o.h/2);ctx.lineTo(x+32,y+o.h/2-16*fl);ctx.lineTo(x+22,y+o.h/2+2);ctx.closePath();ctx.fill();
      ctx.fillStyle='#d8b060';ctx.beginPath();ctx.moveTo(x+24,y+o.h/2-2);ctx.lineTo(x+32,y+o.h/2);ctx.lineTo(x+24,y+o.h/2+3);ctx.closePath();ctx.fill();
    }
  }
}
function drawCoins(){
  for(const c of cns){
    const x=c.x,y=hor-30-c.y,sq=Math.abs(Math.sin(timeMs*0.004+c.ph));
    ctx.save();ctx.translate(x,y);ctx.scale(0.4+sq*0.6,1);
    const g=ctx.createRadialGradient(-3,-3,1,0,0,12);
    g.addColorStop(0,'#fff3b0');g.addColorStop(0.5,'#ffd24a');g.addColorStop(1,'#c8901a');
    ctx.fillStyle=g;ctx.beginPath();ctx.arc(0,0,12,0,7);ctx.fill();
    ctx.strokeStyle='rgba(120,70,10,0.6)';ctx.lineWidth=1.5;ctx.stroke();
    ctx.fillStyle='#8a5a10';ctx.font='bold 11px Georgia';ctx.textAlign='center';ctx.fillText('₳',0,4);
    ctx.restore();
  }
}
function drawPuffs(){
  for(const p of puffs){
    ctx.fillStyle='rgba(160,120,70,'+(p.l*0.5).toFixed(2)+')';
    ctx.beginPath();ctx.arc(p.x,p.y,p.r*(1.6-p.l),0,7);ctx.fill();
  }
}
function drawVignette(){
  const g=ctx.createRadialGradient(W/2,H*0.45,H*0.3,W/2,H*0.5,H*0.85);
  g.addColorStop(0,'rgba(0,0,0,0)');g.addColorStop(1,'rgba(0,0,0,0.35)');
  ctx.fillStyle=g;ctx.fillRect(0,0,W,H);
}

// ---------- game over ----------
let overAt=0;
function gameOver(){
  S='over';overAt=timeMs;
  const sc=Math.floor(score);
  if(sc>hi){hi=sc;localStorage.setItem('lionHi',hi);document.getElementById('hiscore').textContent=hi;}
  document.getElementById('msg').innerHTML='🦁 Le Lion est tombé... Touche pour revivre';
  fetch('/jeux/score',{method:'POST',headers:{'Content-Type':'application/x-www-form-urlencoded'},body:'score='+sc+'&coins='+coins})
    .then(r=>r.json()).then(d=>{if(d.ok)document.getElementById('srv').innerHTML='🪙 Compte: '+d.coins+' pièces · Record: '+d.best;}).catch(()=>{});
}

// ---------- boucle ----------
let last=0;
function loop(ts){
  const dt=Math.min(0.033,(ts-last)/1000||0.016);last=ts;timeMs=ts;
  const sky=skyAt(dayT);
  document.getElementById('phase').textContent=sky.name;

  if(S==='play'){
    speed+=dt*9;
    dayT=(dayT+dt*0.012)%1;
    score+=speed*dt*0.02;
    spawn(dt);
    // physique
    lion.vy+=G*3*dt*0.55;lion.y-=lion.vy*dt;
    if(lion.y<=0){if(lion.vy>300&&lion.jumps>0){for(let i=0;i<8;i++)puffs.push({x:W*0.18+(Math.random()*30-15),y:hor,vx:-40-Math.random()*60,vy:-10-Math.random()*30,l:0.6,r:3+Math.random()*5});}
      lion.y=0;lion.vy=0;lion.jumps=0;}
    lion.ph+=dt*(8+speed*0.012);
    if(lion.y<=0.5&&Math.random()<0.3)puffs.push({x:W*0.18-20,y:hor-2,vx:-80-Math.random()*60,vy:-5-Math.random()*15,l:0.45,r:2+Math.random()*3});
    // mouvements
    for(const o of obs){o.x-=speed*dt*(o.t==='aigle'?1.35:1);}
    for(const c of cns){c.x-=speed*dt;}
    obs=obs.filter(o=>o.x>-80);cns=cns.filter(c=>c.x>-40);
    // collisions
    const lx=W*0.18,ly=hor-lion.y,lb={x:lx-24,y:ly-48,w:52,h:48};
    for(const o of obs){
      const oy=o.t==='aigle'?hor-o.fly-o.h+Math.sin(timeMs*0.004+o.ph)*14:hor-o.h;
      const ob={x:o.x-o.w/2+4,y:oy+4,w:o.w-8,h:o.h-4};
      if(lb.x<ob.x+ob.w&&lb.x+lb.w>ob.x&&lb.y<ob.y+ob.h&&lb.y+lb.h>ob.y){gameOver();break;}
    }
    for(const c of cns){
      if(Math.abs(c.x-lx)<26&&Math.abs((hor-30-c.y)-ly+20)<32){
        c.got=true;coins++;
        for(let i=0;i<7;i++)puffs.push({x:c.x,y:hor-30-c.y,vx:(Math.random()*160-80),vy:-Math.random()*120,l:0.5,r:2+Math.random()*3,gold:true});
        document.getElementById('coins').textContent='🪙 '+coins;
      }
    }
    cns=cns.filter(c=>!c.got);
  }
  for(const p of puffs){p.x+=p.vx*dt;p.y+=p.vy*dt;p.l-=dt*1.6;}
  puffs=puffs.filter(p=>p.l>0);

  // ---------- rendu ----------
  drawSky(sky);
  drawRidge(mtn2,0.12,rgb(lerpC(sky.mid,[10,8,22],0.55+sky.night*0.3)),0);
  drawRidge(mtn1,0.25,rgb(lerpC(sky.mid,[8,6,18],0.75+sky.night*0.2)),H*0.02);
  for(const t of trees){
    const x=((t.x*W*1.6-scroll*0.45)%(W*1.6)+W*1.6)%(W*1.6)-W*0.3;
    acacia(x,hor+4,t.s*0.9,sky);
  }
  drawGround(sky);
  drawObs(sky);
  drawCoins();
  drawPuffs();
  drawLion(sky);
  drawVignette();

  if(S==='menu'){
    ctx.fillStyle='rgba(10,5,0,0.45)';ctx.fillRect(0,0,W,H);
    ctx.fillStyle='#ffe8b0';ctx.font='bold 34px Georgia';ctx.textAlign='center';
    ctx.fillText('🦁 Le Lion du Sahel',W/2,H*0.34);
    ctx.font='16px Georgia';ctx.fillStyle='#f0d8a8';
    ctx.fillText('La Vraie Savane — du soleil doré à la nuit étoilée',W/2,H*0.40);
    ctx.fillText('Touche pour courir 🏃',W/2,H*0.50);
  }
  if(S==='over'){
    ctx.fillStyle='rgba(10,5,0,0.55)';ctx.fillRect(0,H*0.28,W,H*0.44);
    ctx.fillStyle='#ffe8b0';ctx.font='bold 30px Georgia';ctx.textAlign='center';
    ctx.fillText('🦁 Le Lion est tombé',W/2,H*0.38);
    ctx.font='18px Georgia';ctx.fillStyle='#f0d8a8';
    ctx.fillText('Score: '+Math.floor(score)+'  ·  Pièces: '+coins+' 🪙',W/2,H*0.45);
    ctx.fillText('Record: '+hi,W/2,H*0.50);
    ctx.font='15px Georgia';
    ctx.fillText('Touche pour revivre',W/2,H*0.58);
  }
  document.getElementById('score').textContent=Math.floor(score);
  requestAnimationFrame(loop);
}
requestAnimationFrame(loop);
</script>
</body></html>
"##;

fn handle_request(mut stream: TcpStream, state: Arc<Mutex<AppState>>) {
    let mut buffer = [0u8; 65536];
    let mut total = 0;
    
    // Read headers
    loop {
        let n = stream.read(&mut buffer[total..]).unwrap_or(0);
        if n == 0 { break; }
        total += n;
        let header_end = String::from_utf8_lossy(&buffer[..total]).find("\r\n\r\n");
        if header_end.is_some() { break; }
        if total >= buffer.len() - 1 { break; }
    }

    let request_str = String::from_utf8_lossy(&buffer[..total]).to_string();
    let (headers_part, body_part) = request_str.split_once("\r\n\r\n").unwrap_or((&request_str, ""));

    let mut method = "GET";
    let mut path = "/";
    
    if let Some(first_line) = headers_part.lines().next() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            method = parts[0];
            path = parts[1];
        }
    }

    let (clean_path, query_params) = parse_query(path);
    let current_user = {
        let s = state.lock().unwrap();
        get_session_user(headers_part, &s)
    };

    // ===== GET /{owner}/{repo}/download — Telechargement ZIP binaire =====
    if method == "GET" && clean_path.ends_with("/download") && clean_path.matches('/').count() == 3 {
        let parts: Vec<&str> = clean_path.trim_start_matches('/').split('/').collect();
        if parts.len() == 3 {
            let (owner, repo_name) = (parts[0], parts[1]);
            let s = state.lock().unwrap();
            let allowed = match s.find_repo(owner, repo_name) {
                Some(repo) => repo.is_public || current_user.as_deref() == Some(owner),
                None => false,
            };
            if allowed {
                let repo = s.find_repo(owner, repo_name).unwrap();
                let files: Vec<(String, String)> = repo.files.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                let zip_bytes = build_zip(&files);
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/zip\r\nContent-Disposition: attachment; filename=\"{}.zip\"\r\nContent-Length: {}\r\n\r\n",
                    repo_name, zip_bytes.len()
                );
                let mut response = head.into_bytes();
                response.extend_from_slice(&zip_bytes);
                let _ = stream.write_all(&response);
                let _ = stream.flush();
                return;
            }
        }
    }

    let (status, content_type, body) = match (method, clean_path.as_str()) {
        ("GET", "/") => {
            let s = state.lock().unwrap();
            ("200", "text/html; charset=utf-8", html_home(&s, current_user.as_deref()))
        }
        ("GET", "/register") => {
            ("200", "text/html; charset=utf-8", html_register())
        }
        ("POST", "/register") => {
            let form = parse_form(body_part);
            let username = form.get("username").cloned().unwrap_or_default();
            let password = form.get("password").cloned().unwrap_or_default();
            let email = form.get("email").cloned().unwrap_or_default();
            let country = form.get("country").cloned().unwrap_or_default();
            let bio = form.get("bio").cloned().unwrap_or_default();

            let mut s = state.lock().unwrap();
            if s.find_user(&username).is_some() {
                ("200", "text/html; charset=utf-8", html_page("Erreur", "<div style='color:#f85149;'><h1>Nom d'utilisateur deja pris</h1><a href='/register'>Reessayer</a></div>"))
            } else if username.is_empty() || password.is_empty() {
                ("200", "text/html; charset=utf-8", html_page("Erreur", "<div style='color:#f85149;'><h1>Nom d'utilisateur et mot de passe requis</h1><a href='/register'>Reessayer</a></div>"))
            } else {
                s.users.push(User {
                    username: username.clone(),
                    password_hash: simple_hash(&password),
                    email,
                    country,
                    created_at: now_string(),
                    bio,
                });
                s.save();
                let token = gen_token();
                s.sessions.insert(token.clone(), username.clone());
                s.save();
                ("302", "text/html", format!("Set-Cookie: afriforme_session={}; Path=/; HttpOnly\r\nLocation: /", token))
            }
        }
        ("GET", "/login") => {
            ("200", "text/html; charset=utf-8", html_login())
        }
        ("POST", "/login") => {
            let form = parse_form(body_part);
            let username = form.get("username").cloned().unwrap_or_default();
            let password = form.get("password").cloned().unwrap_or_default();
            let hash = simple_hash(&password);

            let mut s = state.lock().unwrap();
            if let Some(_user) = s.find_user(&username) {
                if s.find_user(&username).unwrap().password_hash == hash {
                    let token = gen_token();
                    s.sessions.insert(token.clone(), username);
                    ("302", "text/html", format!("Set-Cookie: afriforme_session={}; Path=/; HttpOnly\r\nLocation: /", token))
                } else {
                    ("200", "text/html; charset=utf-8", html_page("Erreur", "<div style='color:#f85149;'><h1>Mot de passe incorrect</h1><a href='/login'>Reessayer</a></div>"))
                }
            } else {
                ("200", "text/html; charset=utf-8", html_page("Erreur", "<div style='color:#f85149;'><h1>Utilisateur non trouve</h1><a href='/login'>Reessayer</a></div>"))
            }
        }
        ("GET", "/logout") => {
            // Simple: just redirect
            ("302", "text/html", "Set-Cookie: afriforme_session=; Path=/; Max-Age=0\r\nLocation: /".to_string())
        }
        ("GET", "/new") => {
            if current_user.is_none() {
                ("302", "text/html", "Location: /login".to_string())
            } else {
                ("200", "text/html; charset=utf-8", html_new_repo())
            }
        }
        ("POST", "/new") => {
            if let Some(user) = current_user {
                let form = parse_form(body_part);
                let name = form.get("name").cloned().unwrap_or_default();
                let description = form.get("description").cloned().unwrap_or_default();
                let language = form.get("language").cloned().unwrap_or("Rust".to_string());
                let is_public = form.get("visibility").map(|v| v == "public").unwrap_or(true);
                let tags_str = form.get("tags").cloned().unwrap_or_default();
                let tags: Vec<String> = tags_str.split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();

                if !name.is_empty() {
                    let mut s = state.lock().unwrap();
                    let id = s.next_repo_id;
                    s.next_repo_id += 1;
                    s.repos.push(Repository {
                        id,
                        owner: user.clone(),
                        name,
                        description,
                        language,
                        stars: 0,
                        forks: 0,
                        created_at: now_string(),
                        files: HashMap::new(),
                        tags,
                        is_public,
                        views: 0,
                        commits: Vec::new(),
                    });
                    s.save();
                }
                ("302", "text/html", format!("Location: /{}", user))
            } else {
                ("302", "text/html", "Location: /login".to_string())
            }
        }
        ("GET", "/explore") => {
            let s = state.lock().unwrap();
            ("200", "text/html; charset=utf-8", html_explore(&s))
        }
        ("GET", "/afri-net") => {
            ("200", "text/html; charset=utf-8", html_afri_net(current_user.as_deref()))
        }
        ("GET", "/importer-africhain") => {
            // v0.22: copie TOUT notre travail AfriChain (~/afririch/src/*.rs) dans un depot AfriForme
            if let Some(user) = &current_user {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let candidates = vec![
                    format!("{}/afririch/src", home),
                    "../src".to_string(),
                    "./src".to_string(),
                ];
                let src_dir = candidates.into_iter().find(|p| std::path::Path::new(p).exists());
                let mut s = state.lock().unwrap();
                // Trouver un depot existant (majuscules ou non) ou en creer un
                let existing = s.repos.iter().find(|r| r.owner == *user && r.name.eq_ignore_ascii_case("africhain")).map(|r| r.name.clone());
                let repo_name = existing.unwrap_or_else(|| "africhain".to_string());
                if s.find_repo(user, &repo_name).is_none() {
                    let id = s.next_repo_id;
                    s.next_repo_id += 1;
                    s.repos.push(Repository {
                        id,
                        owner: user.clone(),
                        name: repo_name.clone(),
                        description: "La blockchain africaine souveraine — tout notre travail, notre enfant. Zero dependance, Rust std only, 54 pays, Ed25519, AfriHash-256.".to_string(),
                        language: "Rust".to_string(),
                        stars: 0,
                        forks: 0,
                        created_at: now_string(),
                        files: HashMap::new(),
                        tags: vec!["blockchain".to_string(), "africhain".to_string(), "souverainete".to_string(), "rust".to_string()],
                        is_public: true,
                        views: 0,
                        commits: Vec::new(),
                    });
                }
                let mut imported = 0usize;
                if let Some(dir) = src_dir {
                    if let Ok(entries) = std::fs::read_dir(&dir) {
                        let mut paths: Vec<std::path::PathBuf> = entries.filter_map(|e| e.ok())
                            .map(|e| e.path())
                            .filter(|p| p.extension().map(|x| x == "rs").unwrap_or(false))
                            .collect();
                        paths.sort();
                        for p in paths {
                            let fname = match p.file_name() {
                                Some(f) => f.to_string_lossy().to_string(),
                                None => continue,
                            };
                            if let Ok(content) = std::fs::read_to_string(&p) {
                                if let Some(repo) = s.find_repo_mut(user, &repo_name) {
                                    let is_update = repo.files.contains_key(&fname);
                                    repo.files.insert(fname.clone(), content);
                                    repo.commits.push(Commit {
                                        id: repo.commits.len() + 1,
                                        message: if is_update { format!("Mise a jour: {}", fname) } else { format!("Import de {} — notre travail, notre enfant", fname) },
                                        author: user.clone(),
                                        filename: fname,
                                        created_at: now_string(),
                                    });
                                    imported += 1;
                                }
                            }
                        }
                    }
                }
                if imported > 0 {
                    s.save();
                }
                ("302", "text/html", format!("Location: /{}/{}", user, repo_name))
            } else {
                ("302", "text/html", "Location: /login".to_string())
            }
        }
        ("GET", "/api/copilote") => {
            let q = query_params.get("q").cloned().unwrap_or_default();
            let s = state.lock().unwrap();
            let resp = ai_respond(&q, current_user.as_deref().unwrap_or("invite"), &s);
            ("200", "text/html; charset=utf-8", resp)
        }
        ("GET", "/ai") => {
            if let Some(user) = &current_user {
                let s = state.lock().unwrap();
                let chat = s.ai_chats.get(user);
                // Handle query param question
                if let Some(q) = query_params.get("q") {
                    drop(s);
                    let mut s = state.lock().unwrap();
                    let response = ai_respond(q, user, &s);
                    let chat = s.ai_chats.entry(user.clone()).or_insert(AIChat {
                        user: user.clone(),
                        messages: Vec::new(),
                    });
                    chat.messages.push(("user".to_string(), q.clone()));
                    chat.messages.push(("ai".to_string(), response));
                    if chat.messages.len() > 20 {
                        chat.messages = chat.messages.split_off(chat.messages.len() - 20);
                    }
                    s.save();
                    ("302", "text/html", "Location: /ai".to_string())
                } else {
                    ("200", "text/html; charset=utf-8", html_ai_chat(user, chat))
                }
            } else {
                ("302", "text/html", "Location: /login".to_string())
            }
        }
        ("POST", "/ai") => {
            if let Some(user) = &current_user {
                let form = parse_form(body_part);
                let question = form.get("question").cloned().unwrap_or_default();
                if !question.is_empty() {
                    let response = {
                        let s = state.lock().unwrap();
                        ai_respond(&question, user, &s)
                    };
                    let mut s = state.lock().unwrap();
                    let chat = s.ai_chats.entry(user.clone()).or_insert(AIChat {
                        user: user.clone(),
                        messages: Vec::new(),
                    });
                    chat.messages.push(("user".to_string(), question));
                    chat.messages.push(("ai".to_string(), response));
                    if chat.messages.len() > 20 {
                        chat.messages = chat.messages.split_off(chat.messages.len() - 20);
                    }
                    s.save();
                }
                ("302", "text/html", "Location: /ai".to_string())
            } else {
                ("302", "text/html", "Location: /login".to_string())
            }
        }
        ("GET", "/about") => {
            let s = state.lock().unwrap();
            let stats = format!(r#"
<div class="card">
  <h2>🦁 AfriForme — La plateforme africaine de code</h2>
  <p>AfriForme est une plateforme de developpeurs construite <strong>par l'Afrique, pour l'Afrique</strong> — l'equivalent africain de GitHub, mais souverain.</p>
  <p>Construite en <strong>Rust pur, sans aucune dependance externe</strong>. Le fichier Cargo.toml est vide: pas de bibliotheque occidentale, pas de serveur distant, pas de collecte de donnees. Tout le code — serveur HTTP, JSON, sessions, ZIP, Le Griot — est ecrit from scratch.</p>
</div>
<div class="card">
  <h2>💚 Pourquoi la souverainete?</h2>
  <p>Chaque donnee africaine envoyee sur les plateformes occidentales (GitHub, Google, Meta) est une richesse qui quitte le continent. AfriForme garde le savoir-faire africain <strong>en Afrique</strong>: les depots, les cours, les diplomes, les profils — tout reste chez nous.</p>
  <p><em>"Si pour l'Occident c'est noir, pour nous c'est blanc."</em></p>
</div>
<div class="card">
  <h2>✨ Fonctionnalites</h2>
  <div class="stats">
    <div class="stat"><div class="num">📦 {}</div><div class="label">Depots</div></div>
    <div class="stat"><div class="num">👤 {}</div><div class="label">Developpeurs</div></div>
    <div class="stat"><div class="num">🎓 {}</div><div class="label">Cours auto-generees</div></div>
    <div class="stat"><div class="num">🤖 1</div><div class="label">Griot integre</div></div>
  </div>
  <ul>
    <li>📦 Depots de code publics ou prives, avec tags, langages et recherche</li>
    <li>🎓 <strong>Cours auto-generees</strong> — chaque depot devient automatiquement un cours avec exercices et diplome grave</li>
    <li>🌳 Baobabs, 🌿 Boutures, 💬 Commentaires, 🐜 Termites, 🤝 Sanankus</li>
    <li>🏆 Classement des developpeurs, 🔥 depots en tendance</li>
    <li>🔔 Notifications, 📂 Fil d'activite</li>
    <li>⬇ Telechargement ZIP (construit from scratch, zero dependance)</li>
    <li>📖 Le Griot — IA qui repond aux questions techniques</li>
  </ul>
</div>
<div class="card">
  <h2>🗣️ Notre Langage — le vocabulaire africain d'AfriForme</h2>
  <p>AfriForme ne parle pas comme les plateformes occidentales. Chaque mot vient de nos traditions:</p>
  <ul>
    <li>🌳 <strong>Baobab</strong> — honorer un depot (au lieu de "star"). On plante un baobab sur le travail qu'on respecte. L'arbre de vie qui nourrit le village.</li>
    <li>🌿 <strong>Bouture</strong> — copier un depot pour le faire grandir (au lieu de "fork"). Une bouture devient son propre arbre, mais sa racine reste la meme.</li>
    <li>🐜 <strong>Termite</strong> — un probleme dans le code (au lieu de "issue"). Le termite attaque la case silencieusement — on le signale avant qu'il ne detruise tout.</li>
    <li>🤝 <strong>Sananku</strong> — suivre un developpeur (au lieu de "follow"). Le cousinage a plaisanterie: un lien sacre ouest-africain entre familles.</li>
    <li>🔥 <strong>La Braise</strong> — les depots en pleine activite (au lieu de "trending"). Ce qui brule maintenant dans la forge.</li>
    <li>🏛️ <strong>Conseil des Sages</strong> — le classement des developpeurs. Chez nous, ce sont les anciens qui guident.</li>
    <li>🥁 <strong>Le Tambour</strong> — les notifications. Le tambour parleur portait les messages a travers la savane.</li>
    <li>📖 <strong>Le Griot</strong> — l'IA assistante. Le griot garde la memoire du village et transmet le savoir.</li>
    <li>🦁 <strong>La Savane</strong> — l'exploration des depots. La savane ou vivent tous les projets.</li>
  </ul>
  <p><em>Le langage est souverainete: penser en ses propres mots, c'est exister en son propre nom.</em></p>
</div>
<div class="card">
  <h2>🏗️ Architecture</h2>
  <ul>
    <li><strong>Serveur HTTP</strong> — ecrit from scratch sur TcpListener (pas de framework)</li>
    <li><strong>JSON</strong> — parseur et serialiseur maison</li>
    <li><strong>Sessions</strong> — cookies et authentiation maison</li>
    <li><strong>ZIP</strong> — constructeur de fichiers .zip maison (CRC32 inclus)</li>
    <li><strong>Zero dependance</strong> — Cargo.toml vide, Rust std only</li>
  </ul>
</div>
<div class="card">
  <h2>👤 Createur</h2>
  <p><strong>Koffi Christ Olivier</strong> — developpeur africain. AfriForme est construit ligne par ligne, avec la conviction que <strong>l'Afrique est le continent le plus riche</strong> et que sa technologie doit lui appartenir.</p>
</div>
"#, s.repos.len(), s.users.len(), s.courses.len());
            drop(s);
            ("200", "text/html; charset=utf-8", html_page("A propos", &stats))
        }
        ("GET", "/ecole") => {
            let levels = ecole_levels();
            let (progress, is_logged) = {
                let s = state.lock().unwrap();
                match &current_user {
                    Some(u) => (s.ecole_progress.get(u).cloned().unwrap_or_default(), true),
                    None => (Vec::new(), false),
                }
            };
            let mut body = String::new();
            body.push_str(r#"<div class="card" style="text-align:center;">
<h1>🌱 L'Ecole du Village</h1>
<p style="font-size:1.1em;">Le savoir africain, du CP1 a la Terminale.</p>
<p>Notre ecole ne porte pas les noms occidentaux (CFEE, BEPC, BAC). Nos diplomes viennent de <strong>notre propre culture</strong> — la semence, le griot, le baobab.</p>
<p style="color:#f59e0b;"><em>"On n'a vraiment appris que ce qu'on peut enseigner."</em></p>
"#);
            if !is_logged {
                body.push_str("<p style='color:#f85149;'>Connecte-toi pour suivre les lecons et passer les niveaux. <a href='/login'>Connexion</a> · <a href='/register'>S'inscrire</a></p>");
            }
            body.push_str("</div>");

            for cycle in ["Semence", "Griot", "Baobab", "Sage", "Science", "Maths", "Techno"] {
                let cycle_levels: Vec<&EcoleLevel> = levels.iter().filter(|l| l.cycle == cycle).collect();
                let (cycle_name, cycle_emoji, dip_name, dip_desc) = match cycle {
                    "Semence" => ("Cycle de la Semence — Primaire (CP1 → CM2)", "🌱", "🌱 DIPLOME DE LA SEMENCE", "La graine est plantee, elle a germe. L'enfant connait sa terre, ses langues et ses empires."),
                    "Griot" => ("Cycle du Griot — College (6eme → 3eme)", "📖", "📖 DIPLOME DU GRIOT", "Le jeune connait les histoires et peut les transmettre. Il garde la memoire du village."),
                    "Baobab" => ("Cycle du Baobab — Lycee (2nde → Terminale)", "🌳", "🌳 DIPLOME DU BAOBAB", "L'arbre de la sagesse. Le diplome peut batir — code, economie, leadership — et guider les plus jeunes."),
                    "Sage" => ("Cycle du Sage — Universite (L1 → Doctorat)", "🎓", "🎓 DIPLOME DU SAGE", "Le Sage a appris et maintenant il enseigne. Il retourne au village et transmet la sagesse africaine."),
                    "Science" => ("Faculte des Sciences — La matiere, la vie, les etoiles", "🔬", "🔬 DIPLOME DU SAVANT", "Le Savant applique la science a l'Afrique: physique, biologie, astronomie. Comme Cheikh Anta Diop, il prouve par la science."),
                    "Maths" => ("Faculte des Mathematiques — D'Ishango aux fractales", "➗", "➗ DIPLOME DU CALCULATEUR", "Le Calculateur herite d'Ishango: nombres, geometrie, logique. Les maths de l'Afrique, de l'os au blockchain."),
                    _ => ("Faculte de Technologie — Electricite, reseaux, intelligence", "⚙️", "⚙️ DIPLOME DE L'INGENIEUR", "L'Ingenieur construit: circuits solaires, reseaux mesh, IA souveraine. La technologie africaine entre ses mains."),
                };
                let done = ecole_cycle_complete(&progress, cycle);
                let dip_style = if done { "background:#1a2b1a;border:1px solid #238636;" } else { "background:#21262d;border:1px solid #30363d;" };
                body.push_str(&format!("<div class='card'><h2>{} {}</h2>", cycle_emoji, cycle_name));
                for l in &cycle_levels {
                    let completed = progress.iter().any(|p| p == l.slug);
                    let mark = if completed { "✅" } else { "⬜" };
                    body.push_str(&format!("<div class='repo'><h3>{} <a href='/ecole/{}'>{}</a></h3><div class='meta'>{} niveau · {}</div></div>",
                        mark, l.slug, l.title, cycle_emoji, if completed { "Complete" } else { "A suivre" }));
                }
                body.push_str(&format!("<div style='{}border-radius:8px;padding:15px;margin-top:10px;'><h3>{}</h3><p>{}</p><p>{}</p></div></div>",
                    dip_style, dip_name, dip_desc,
                    if done { "🎉 <strong>Diplome obtenu! Bravo, l'Afrique est fiere de toi.</strong>" } else if !is_logged { "Connecte-toi et complete tous les niveaux du cycle pour recevoir ce diplome." } else { "Complete tous les niveaux du cycle pour recevoir ce diplome." }));
            }

            body.push_str(r#"<div class="card">
<h2>📜 La Charte de l'Ecole du Village</h2>
<ul>
<li>L'enfant africain apprend <strong>d'abord sa culture</strong>, ensuite le monde</li>
<li>Les langues africaines sont des matieres, pas des curiosites</li>
<li>L'histoire enseignee est la vraie: les empires avant la colonisation, les resistances pendant, la souverainete apres</li>
<li>Le diplome ne couronne pas la memorisation, mais la transmission</li>
<li>Les reponses des exercices sont gardees par Le Griot — on ne triche pas au village, on apprend</li>
</ul>
</div>"#);
            ("200", "text/html; charset=utf-8", html_page("Ecole du Village", &body))
        }
        ("GET", "/ecole/") => ("302", "text/html", "Location: /ecole".to_string()),
        (m, p) if m == "GET" && p.starts_with("/ecole/") => {
            let slug = p.trim_start_matches("/ecole/").trim_end_matches('/');
            let levels = ecole_levels();
            let level = levels.iter().find(|l| l.slug == slug);
            match level {
                None => ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Ce niveau n'existe pas a l'Ecole du Village.</div>")),
                Some(lv) => {
                    let (completed, is_logged) = {
                        let s = state.lock().unwrap();
                        match &current_user {
                            Some(u) => (s.ecole_progress.get(u).map(|v| v.iter().any(|p| p == lv.slug)).unwrap_or(false), true),
                            None => (false, false),
                        }
                    };
                    let mut body = String::new();
                    body.push_str(&format!("<div class='card'><h1>{} {}</h1><p>{} · <a href='/ecole'>← Ecole du Village</a></p>{}</div>",
                        if completed { "✅" } else { "⬜" }, lv.title, lv.cycle, lv.lessons));
                    body.push_str(&format!("<div class='card'><h2>📝 Exercices du niveau</h2>"));
                    if !is_logged {
                        body.push_str("<p style='color:#f85149;'>Connecte-toi pour passer les exercices. <a href='/login'>Connexion</a></p>");
                    } else if completed {
                        body.push_str("<p style='color:#238636;'>✅ Niveau complete! Passe au niveau suivant.</p>");
                    } else {
                        body.push_str(&format!("<form method='POST' action='/ecole/{}/repondre'>", lv.slug));
                        for (i, (q, _, _)) in lv.exercises.iter().enumerate() {
                            body.push_str(&format!("<p style='margin:12px 0;'><strong>Question {}:</strong> {}<br><input type='text' name='a{}' placeholder='Ta reponse...' style='width:60%;margin-top:5px;'></p>", i + 1, q, i + 1));
                        }
                        body.push_str("<button type='submit' class='btn'>📖 Le Griot evalue mes reponses</button></form>");
                        body.push_str("<p style='color:#8b949e;font-size:0.85em;margin-top:10px;'>Les reponses sont gardees par Le Griot. Apres ta soumission, il t'explique chaque reponse: le POURQUOI, le COMMENT, et le VOILA — comme les anciens autour du feu.</p>");
                    }
                    body.push_str("</div>");
                    ("200", "text/html; charset=utf-8", html_page(lv.title, &body))
                }
            }
        }
        (m, p) if m == "POST" && p.starts_with("/ecole/") && p.ends_with("/repondre") => {
            let form = parse_form(body_part);
            let user = match &current_user { Some(u) => u.clone(), None => {
                let resp = "HTTP/1.1 302 Found\r\nLocation: /login\r\n\r\n".to_string();
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
                return;
            } };
            let slug = p.trim_start_matches("/ecole/").trim_end_matches("/repondre").to_string();
            let levels = ecole_levels();
            let level = levels.iter().find(|l| l.slug == slug);
            match level {
                None => ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Niveau inconnu.</div>")),
                Some(lv) => {
                    let mut correct = 0;
                    let mut results = String::new();
                    for (i, (_q, a, e)) in lv.exercises.iter().enumerate() {
                        let given = form.get(&format!("a{}", i + 1)).cloned().unwrap_or_default();
                        let ok = ecole_normalize(&given) == ecole_normalize(a);
                        if ok { correct += 1; }
                        let bonne = if ok { String::new() } else { format!("<br>La bonne reponse: <strong>{}</strong>", a) };
                        results.push_str(&format!("<li>{} <strong>Question {}:</strong> {}{}<br><span style=\"color:#8b949e;font-size:0.9em;\">{}</span></li>",
                            if ok { "✅" } else { "❌" }, i + 1, if ok { "juste!" } else { "a revoir" }, bonne, e));
                    }
                    let all_ok = correct == lv.exercises.len();
                    let mut body = String::new();
                    body.push_str(&format!("<div class='card'><h1>{} {} — Resultat: {}/{}</h1><ul>{}</ul>{}</div>",
                        if all_ok { "🎉" } else { "📖" }, lv.title, correct, lv.exercises.len(), results,
                        if all_ok { "<p style='color:#238636;font-size:1.1em;'><strong>Niveau complete! Le Griot grave ta reussite.</strong></p>" } else { "<p style='color:#f85149;'>Le Griot t'invite a reviser les lecons et a reessayer — au village, on apprend jusqu'a reussir.</p>" }));
                    body.push_str(&format!("<div class='card'><a href='/ecole/{}' class='btn btn-secondary'>← Reviser les lecons</a> <a href='/ecole' class='btn'>Ecole du Village</a></div>", lv.slug));
                    if all_ok {
                        let mut s = state.lock().unwrap();
                        let entry = s.ecole_progress.entry(user.clone()).or_default();
                        if !entry.iter().any(|p| p == &lv.slug) { entry.push(lv.slug.to_string()); }
                        s.add_notification(&user, &format!("Niveau {} complete a l'Ecole du Village", lv.title), "/ecole");
                        let cycle_done = ecole_cycle_complete(&s.ecole_progress.get(&user).cloned().unwrap_or_default(), lv.cycle);
                        if cycle_done {
                            let dip = match lv.cycle { "Semence" => "🌱 Diplome de la Semence", "Griot" => "📖 Diplome du Griot", "Baobab" => "🌳 Diplome du Baobab", "Sage" => "🎓 Diplome du Sage", "Science" => "🔬 Diplome du Savant", "Maths" => "➗ Diplome du Calculateur", _ => "⚙️ Diplome de l'Ingenieur" };
                            s.add_notification(&user, &format!("DIPLOME OBTENU: {} — l'Afrique est fiere de toi!", dip), "/ecole");
                        }
                        s.save();
                    }
                    ("200", "text/html; charset=utf-8", html_page("Resultat", &body))
                }
            }
        }
        ("GET", "/courses") => {
            let s = state.lock().unwrap();
            ("200", "text/html; charset=utf-8", html_course_catalog(&s, current_user.as_deref()))
        }
        ("POST", "/profile/edit") => {
            if let Some(user) = &current_user {
                let form = parse_form(body_part);
                let bio = form.get("bio").cloned().unwrap_or_default();
                let mut s = state.lock().unwrap();
                if let Some(u) = s.users.iter_mut().find(|u| u.username == *user) {
                    u.bio = bio;
                    s.save();
                }
                ("302", "text/html", format!("Location: /{}", user))
            } else {
                ("302", "text/html", "Location: /login".to_string())
            }
        }
        ("GET", "/leaderboard") => {
            let s = state.lock().unwrap();
            ("200", "text/html; charset=utf-8", html_leaderboard(&s))
        }
        ("GET", "/honneur") => {
            let s = state.lock().unwrap();
            ("200", "text/html; charset=utf-8", html_honneur(&s))
        }
        ("GET", "/jeux") => {
            ("200", "text/html; charset=utf-8", GAME_HTML.to_string())
        }
        ("GET", "/jeux/score") => {
            match current_user.as_deref() {
                Some(user) => {
                    let s = state.lock().unwrap();
                    let g = s.game_scores.get(user).cloned().unwrap_or(GameScore { coins: 0, best: 0 });
                    ("200", "application/json", format!("{{\"ok\":true,\"coins\":{},\"best\":{}}}", g.coins, g.best))
                }
                None => ("200", "application/json", "{\"ok\":false}".to_string()),
            }
        }
        ("POST", "/jeux/score") => {
            let form = parse_form(body_part);
            let score: u32 = form.get("score").and_then(|v| v.parse().ok()).unwrap_or(0);
            let coins: u32 = form.get("coins").and_then(|v| v.parse().ok()).unwrap_or(0);
            match current_user.as_deref() {
                Some(user) => {
                    let mut s = state.lock().unwrap();
                    let entry = s.game_scores.entry(user.to_string()).or_insert(GameScore { coins: 0, best: 0 });
                    entry.coins += coins;
                    if score > entry.best { entry.best = score; }
                    let (tc, tb) = (entry.coins, entry.best);
                    s.save();
                    ("200", "application/json", format!("{{\"ok\":true,\"coins\":{},\"best\":{}}}", tc, tb))
                }
                None => ("200", "application/json", "{\"ok\":false}".to_string()),
            }
        }
        (m, p) if m == "GET" && p.starts_with("/diplome/") => {
            let cycle = p.trim_start_matches("/diplome/").to_string();
            match current_user.as_deref() {
                Some(user) => {
                    let s = state.lock().unwrap();
                    let progress = s.ecole_progress.get(user).cloned().unwrap_or_default();
                    if ecole_cycle_complete(&progress, &cycle) {
                        ("200", "text/html; charset=utf-8", html_diplome_cert(user, &cycle))
                    } else {
                        ("200", "text/html; charset=utf-8", html_page("Diplome", "<div class='card'><h1>🔒 Pas encore</h1><p>Ce diplome n'est pas encore gagne. Retourne a l'<a href='/ecole'>Ecole du Village</a> et complete le cycle.</p></div>"))
                    }
                }
                None => ("200", "text/html; charset=utf-8", html_page("Diplome", "<div class='card'><h1>🔒 Connecte-toi</h1><p>Les diplomes sont personnels. <a href='/login'>Connexion</a></p></div>")),
            }
        }
        ("GET", "/search") => {
            let s = state.lock().unwrap();
            let q = query_params.get("q").cloned().unwrap_or_default();
            ("200", "text/html; charset=utf-8", html_search(&s, current_user.as_deref(), &q))
        }
        ("GET", "/notifications") => {
            let s = state.lock().unwrap();
            ("200", "text/html; charset=utf-8", html_notifications(&s, current_user.as_deref()))
        }
        ("POST", "/notifications/read") => {
            if let Some(user) = &current_user {
                let mut s = state.lock().unwrap();
                s.mark_notifications_read(user);
                s.save();
                ("302", "text/html", "Location: /notifications".to_string())
            } else {
                ("302", "text/html", "Location: /login".to_string())
            }
        }
        // User profile route — single segment (before catch-all)
        (m, p) if m == "GET" && p.starts_with('/') && p.matches('/').count() == 1 && p.len() > 1 => {
            let username = &p[1..];
            let s = state.lock().unwrap();
            if let Some(user) = s.find_user(username) {
                let user_clone = user.clone();
                drop(s);
                let s2 = state.lock().unwrap();
                ("200", "text/html; charset=utf-8", html_user_profile(&user_clone, &s2, current_user.as_deref()))
            } else {
                ("404", "text/html; charset=utf-8", html_page("404", &format!("<div class='empty'>Utilisateur '{}' non trouve</div>", username)))
            }
        }
        // Course routes — must be before catch-all patterns
        (m, p) if m == "GET" && p.starts_with("/course/") => {
            // GET /course/{owner}/{repo}
            let parts: Vec<&str> = p.trim_start_matches('/').splitn(4, '/').collect();
            // parts: ["course", owner, repo, ...]
            if parts.len() >= 3 {
                let owner = parts[1];
                let repo_name = parts[2];
                let mut s = state.lock().unwrap();
                if let Some(repo) = s.find_repo(owner, repo_name) {
                    let repo_id = repo.id;
                    let repo_clone = repo.clone();
                    // Auto-generate course if not exists and repo has files
                    if s.find_course_by_repo(repo_id).is_none() && !repo_clone.files.is_empty() {
                        let course = generate_course(&repo_clone);
                        s.courses.push(course);
                        s.save();
                    }
                    if let Some(course) = s.find_course_by_repo(repo_id) {
                        let course_clone = course.clone();
                        drop(s);
                        ("200", "text/html; charset=utf-8", html_course(&course_clone, &repo_clone, owner, repo_name, current_user.as_deref()))
                    } else {
                        ("200", "text/html; charset=utf-8", html_page("Cours", &format!(
                            "<div class='empty'><h1>🎓 Cours non disponible</h1><p>Aucun fichier dans ce depot. Ajoute du code pour generer un cours automatiquement!</p><p><a href='/{}/{}'>Retour au depot</a></p></div>",
                            owner, repo_name
                        )))
                    }
                } else {
                    ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Depot non trouve</div>"))
                }
            } else {
                ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Page non trouvee</div>"))
            }
        }
        (m, p) if m == "POST" && p.starts_with("/course/") => {
            // POST /course/{owner}/{repo}/answer
            let parts: Vec<&str> = p.trim_start_matches('/').splitn(5, '/').collect();
            // parts: ["course", owner, repo, "answer", ...]
            if parts.len() >= 4 && parts[3] == "answer" {
                let owner = parts[1];
                let repo_name = parts[2];
                if let Some(user) = &current_user {
                    let form = parse_form(body_part);
                    let ex_index: usize = form.get("ex_index").and_then(|s| s.parse().ok()).unwrap_or(0);
                    let user_answer = form.get("answer").cloned().unwrap_or_default();
                    
                    let mut s = state.lock().unwrap();
                    if let Some(repo) = s.find_repo(owner, repo_name) {
                        let repo_id = repo.id;
                        if let Some(course) = s.find_course_by_repo_mut(repo_id) {
                            if ex_index < course.exercises.len() {
                                let correct_answer = &course.exercises[ex_index].answer;
                                // Case-insensitive, trimmed comparison
                                let is_correct = user_answer.trim().to_lowercase() == correct_answer.trim().to_lowercase();
                                if is_correct {
                                    // Mark exercise as completed
                                    let progress_str = course.progress.entry(user.clone()).or_insert_with(String::new);
                                    let mut completed: Vec<usize> = progress_str.split(',')
                                        .filter_map(|n| n.parse::<usize>().ok()).collect();
                                    if !completed.contains(&ex_index) {
                                        completed.push(ex_index);
                                    }
                                    *progress_str = completed.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
                                    s.save();
                                }
                            }
                        }
                    }
                    ("302", "text/html", format!("Location: /course/{}/{}", owner, repo_name))
                } else {
                    ("302", "text/html", "Location: /login".to_string())
                }
            } else {
                ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Action non trouvee</div>"))
            }
        }
        (m, p) if p.starts_with('/') && p.matches('/').count() >= 2 && m == "GET" => {
            // /owner/repo pattern
            let parts: Vec<&str> = p.trim_start_matches('/').splitn(3, '/').collect();
            if parts.len() >= 2 {
                let owner = parts[0];
                let repo_name = parts[1];
                let sub = if parts.len() >= 3 { parts[2] } else { "" };

                // Compteur de vues (le proprietaire ne compte pas)
                {
                    let mut sv = state.lock().unwrap();
                    let is_owner_v = current_user.as_deref() == Some(owner);
                    if let Some(repo) = sv.find_repo_mut(owner, repo_name) {
                        if (repo.is_public || is_owner_v) && sub.is_empty() {
                            repo.views += 1;
                            sv.save();
                        }
                    }
                }

                let s = state.lock().unwrap();
                if let Some(repo) = s.find_repo(owner, repo_name) {
                    let is_owner = current_user.as_deref() == Some(owner);
                    if !repo.is_public && !is_owner {
                        // Depot prive: invisible pour les autres
                        ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Depot non trouve</div>"))
                    } else if sub.starts_with("file/") {
                        let filename = &sub[5..];
                        if let Some(content) = repo.files.get(filename) {
                            ("200", "text/html; charset=utf-8", html_file_view(repo, owner, repo_name, filename, content, is_owner))
                        } else {
                            ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Fichier non trouve</div>"))
                        }
                    } else if sub == "upload" && is_owner {
                        ("200", "text/html; charset=utf-8", html_upload(owner, repo_name))
                    } else if sub == "settings" && is_owner {
                        let vis_checked_pub = if repo.is_public { "checked" } else { "" };
                        let vis_checked_priv = if !repo.is_public { "checked" } else { "" };
                        ("200", "text/html; charset=utf-8", html_page(&format!("Parametres — {}/{}", owner, repo_name), &format!(r#"
<div class="card">
  <h2>⚙ Parametres du depot</h2>
  <form method="POST" action="/{}/{}/settings">
    <p><label>Description</label><br><input type="text" name="description" value="{}" style="width:90%;"></p>
    <p><label>Visibilite</label><br>
      <input type="radio" name="visibility" value="public" {}> Public
      <input type="radio" name="visibility" value="private" {}> Prive
    </p>
    <button type="submit" class="btn">Enregistrer</button>
  </form>
</div>
<div class="card" style="border:1px solid #f85149;">
  <h2 style="color:#f85149;">⚠ Zone dangereuse</h2>
  <form method="POST" action="/{}/{}/settings" onsubmit="return confirm('Supprimer ce depot definitivement?');">
    <input type="hidden" name="op" value="delete">
    <button type="submit" class="btn btn-danger">Supprimer ce depot</button>
  </form>
</div>
"#, owner, repo_name, repo.description, vis_checked_pub, vis_checked_priv, owner, repo_name)))
                    } else if sub.is_empty() {
                        let repo_id = repo.id;
                        let repo_comments: Vec<&Comment> = s.get_repo_comments(repo_id);
                        let comments_html = if repo_comments.is_empty() {
                            format!(r#"<h2>💬 Commentaires</h2><div class='empty'>Aucun commentaire. Sois le premier a repondre!</div><form method="POST" action="/{}/{}/comment" style="margin:10px 0;"><input type="text" name="text" placeholder="Ecrire un commentaire..." style="width:70%;"><button type="submit">Envoyer</button></form>"#, owner, repo_name)
                        } else {
                            let mut html = format!("<h2>💬 Commentaires ({})</h2>", repo_comments.len());
                            for c in &repo_comments {
                                html.push_str(&format!(
                                    r#"<div class="card" style="padding:10px;margin:5px 0;"><strong>{}</strong> <span style="color:#8b949e;font-size:0.8em;">· {}</span><br>{}</div>"#,
                                    c.author, ts_lisible(&c.created_at), c.text
                                ));
                            }
                            if current_user.is_some() {
                                html.push_str(&format!(r#"<form method="POST" action="/{}/{}/comment" style="margin:10px 0;"><input type="text" name="text" placeholder="Ecrire un commentaire..." style="width:70%;"><button type="submit">Envoyer</button></form>"#, owner, repo_name));
                            }
                            html
                        };
                        let repo_issues: Vec<&Issue> = s.get_repo_issues(repo_id);
                        let open_count = repo_issues.iter().filter(|i| i.status == "open").count();
                        let mut issues_html = format!("<h2>🐜 Termites actifs ({})</h2>", open_count);
                        if repo_issues.is_empty() {
                            issues_html.push_str("<div class='empty'>Aucun termite. Ce depot est sain!</div>");
                        } else {
                            for i in &repo_issues {
                                let status = if i.status == "open" { "<span class=\"badge\" style=\"background:#f85149;color:#fff;\">Ouverte</span>" } else { "<span class=\"badge\" style=\"background:#238636;color:#fff;\">Fermee</span>" };
                                issues_html.push_str(&format!(
                                    r#"<div class="card" style="padding:10px;margin:5px 0;">{} <strong>#{}</strong> {} <span style="color:#8b949e;font-size:0.8em;">par {} · {}</span><br>{}<form method="POST" action="/{}/{}/issue/{}/toggle" style="display:inline;"><button type="submit" class="btn btn-secondary" style="font-size:0.7em;">{}</button></form></div>"#,
                                    status, i.id, i.title, i.author, i.created_at, i.body, owner, repo_name, i.id,
                                    if i.status == "open" { "Fermer" } else { "Reouvrir" }
                                ));
                            }
                        }
                        if current_user.is_some() {
                            issues_html.push_str(&format!(r#"<form method="POST" action="/{}/{}/issue" style="margin:10px 0;"><input type="text" name="title" placeholder="Titre du termite..." style="width:45%;"><input type="text" name="body" placeholder="Details..." style="width:45%;"><button type="submit">Signaler</button></form>"#, owner, repo_name));
                        }
                        drop(s);
                        let s2 = state.lock().unwrap();
                        let repo2 = s2.find_repo(owner, repo_name).unwrap();
                        ("200", "text/html; charset=utf-8", html_repo_view(repo2, owner, repo_name, is_owner, current_user.as_deref(), &comments_html, &issues_html))
                    } else {
                        ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Page non trouvee</div>"))
                    }
                } else {
                    ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Depot non trouve</div>"))
                }
            } else {
                ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Page non trouvee</div>"))
            }
        }
        (m, p) if m == "POST" && p.starts_with("/follow/") => {
            // POST /follow/{username}
            let target = p.trim_start_matches("/follow/").trim_end_matches('/');
            if target.is_empty() || target.contains('/') {
                ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Page non trouvee</div>"))
            } else if let Some(user) = &current_user {
                if user == target {
                    ("302", "text/html", format!("Location: /{}", target))
                } else {
                    let mut s = state.lock().unwrap();
                    let was_following = s.is_following(user, target);
                    s.toggle_follow(user, target);
                    if !was_following {
                        s.add_notification(target, &format!("{} est devenu ton sananku", user), &format!("/{}", user));
                    }
                    s.save();
                    ("302", "text/html", format!("Location: /{}", target))
                }
            } else {
                ("302", "text/html", "Location: /login".to_string())
            }
        }
        (m, p) if m == "POST" && p.matches('/').count() >= 2 => {
            // POST /owner/repo/upload, /owner/repo/delete/file, /owner/repo/star
            let parts: Vec<&str> = p.trim_start_matches('/').splitn(4, '/').collect();
            if parts.len() >= 3 {
                let owner = parts[0];
                let repo_name = parts[1];
                let action = parts[2];
                let is_owner = current_user.as_deref() == Some(owner);

                if action == "star" {
                    if let Some(user) = &current_user {
                        let mut s = state.lock().unwrap();
                        if let Some(repo) = s.find_repo(owner, repo_name) {
                            let repo_id = repo.id;
                            let repo_owner = repo.owner.clone();
                            s.toggle_star(user, repo_id);
                            if user != &repo_owner {
                                s.add_notification(&repo_owner, &format!("{} a plante un baobab sur {}/{}", user, owner, repo_name), &format!("/{}/{}", owner, repo_name));
                            }
                            s.save();
                        }
                        ("302", "text/html", format!("Location: /{}/{}", owner, repo_name))
                    } else {
                        ("302", "text/html", "Location: /login".to_string())
                    }
                } else if action == "fork" {
                    if let Some(user) = &current_user {
                        let mut s = state.lock().unwrap();
                        let repo_owner = s.find_repo(owner, repo_name).map(|r| r.owner.clone());
                        s.fork_repo(owner, repo_name, user);
                        if let Some(ro) = repo_owner {
                            if user != &ro {
                                s.add_notification(&ro, &format!("{} a fait une bouture de {}/{}", user, owner, repo_name), &format!("/{}/fork-{}", user, repo_name));
                            }
                        }
                        s.save();
                        ("302", "text/html", format!("Location: /{}/fork-{}", user, repo_name))
                    } else {
                        ("302", "text/html", "Location: /login".to_string())
                    }
                } else if action == "comment" {
                    if let Some(user) = &current_user {
                        let form = parse_form(body_part);
                        let text = form.get("text").cloned().unwrap_or_default();
                        if !text.is_empty() {
                            let mut s = state.lock().unwrap();
                            if let Some(repo) = s.find_repo(owner, repo_name) {
                                let repo_id = repo.id;
                                s.comments.push(Comment {
                                    repo_id,
                                    author: user.clone(),
                                    text,
                                    created_at: now_string(),
                                });
                                s.save();
                            }
                        }
                        ("302", "text/html", format!("Location: /{}/{}", owner, repo_name))
                    } else {
                        ("302", "text/html", "Location: /login".to_string())
                    }
                } else if action == "issue" {
                    if let Some(user) = &current_user {
                        let form = parse_form(body_part);
                        if let Some(rest) = parts.get(3) {
                            // POST /owner/repo/issue/{id}/toggle
                            let sub_parts: Vec<&str> = rest.split('/').collect();
                            if sub_parts.len() == 2 && sub_parts[1] == "toggle" {
                                if let Ok(issue_id) = sub_parts[0].parse::<usize>() {
                                    let mut s = state.lock().unwrap();
                                    s.toggle_issue_status(issue_id);
                                    s.save();
                                }
                            }
                        } else {
                            let title = form.get("title").cloned().unwrap_or_default();
                            let body_text = form.get("body").cloned().unwrap_or_default();
                            if !title.is_empty() {
                                let mut s = state.lock().unwrap();
                                if let Some(repo) = s.find_repo(owner, repo_name) {
                                    let repo_id = repo.id;
                                    s.add_issue(repo_id, user, &title, &body_text);
                                    s.save();
                                }
                            }
                        }
                        ("302", "text/html", format!("Location: /{}/{}", owner, repo_name))
                    } else {
                        ("302", "text/html", "Location: /login".to_string())
                    }
                } else if action == "settings" {
                    if !is_owner {
                        ("403", "text/html; charset=utf-8", html_page("403", "<div class='empty'>Acces refuse</div>"))
                    } else {
                        let form = parse_form(body_part);
                        let op = form.get("op").cloned().unwrap_or_default();
                        if op == "delete" {
                            let mut s = state.lock().unwrap();
                            s.delete_repo(owner, repo_name);
                            s.save();
                            ("302", "text/html", format!("Location: /{}", owner))
                        } else {
                            let description = form.get("description").cloned().unwrap_or_default();
                            let is_public = form.get("visibility").map(|v| v == "public").unwrap_or(true);
                            let mut s = state.lock().unwrap();
                            s.update_repo_settings(owner, repo_name, &description, is_public);
                            s.save();
                            ("302", "text/html", format!("Location: /{}/{}", owner, repo_name))
                        }
                    }
                } else if !is_owner {
                    ("403", "text/html; charset=utf-8", html_page("403", "<div class='empty'>Acces refuse</div>"))
                } else if action == "upload" {
                    let form = parse_form(body_part);
                    let filename = form.get("filename").cloned().unwrap_or_default();
                    let content = form.get("content").cloned().unwrap_or_default();
                    if !filename.is_empty() {
                        let mut s = state.lock().unwrap();
                        let repo_id;
                        let repo_clone;
                        let commit_message = form.get("commit_message").cloned()
                            .filter(|m| !m.is_empty())
                            .unwrap_or_else(|| format!("Ajout de {}", filename));
                        if let Some(repo) = s.find_repo_mut(owner, repo_name) {
                            let is_update = repo.files.contains_key(&filename);
                            repo.files.insert(filename.clone(), content);
                            let commit = Commit {
                                id: repo.commits.len() + 1,
                                message: if is_update { format!("Mise a jour: {}", commit_message) } else { commit_message },
                                author: owner.to_string(),
                                filename,
                                created_at: now_string(),
                            };
                            repo.commits.push(commit);
                            repo_id = repo.id;
                            repo_clone = repo.clone();
                        } else {
                            return;
                        }
                        // Auto-generate course if none exists (outside mutable borrow)
                        if s.find_course_by_repo(repo_id).is_none() && !repo_clone.files.is_empty() {
                            let course = generate_course(&repo_clone);
                            s.courses.push(course);
                        }
                        s.save();
                    }
                    ("302", "text/html", format!("Location: /{}/{}", owner, repo_name))
                } else if action == "delete" && parts.len() >= 4 {
                    let filename = parts[3];
                    let mut s = state.lock().unwrap();
                    if let Some(repo) = s.find_repo_mut(owner, repo_name) {
                        repo.files.remove(filename);
                        repo.commits.push(Commit {
                            id: repo.commits.len() + 1,
                            message: format!("Suppression de {}", filename),
                            author: owner.to_string(),
                            filename: filename.to_string(),
                            created_at: now_string(),
                        });
                        s.save();
                    }
                    ("302", "text/html", format!("Location: /{}/{}", owner, repo_name))
                } else {
                    ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Action non trouvee</div>"))
                }
            } else {
                ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Action non trouvee</div>"))
            }
        }
        _ => {
            ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Page non trouvee</div>"))
        }
    };

    let response = if status == "302" {
        format!(
            "HTTP/1.1 302 Found\r\n{}\r\nContent-Length: 0\r\n\r\n",
            body
        )
    } else {
        let status_line = match status {
            "403" => "HTTP/1.1 403 Forbidden",
            "404" => "HTTP/1.1 404 Not Found",
            _ => "HTTP/1.1 200 OK",
        };
        format!(
            "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            content_type,
            body.len(),
            body
        )
    };

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

// Workaround for early return in match — use a helper
impl AppState {}

// ============================================================
// MAIN
// ============================================================

fn main() {
    let port = 8091;
    let state = Arc::new(Mutex::new(AppState::new()));

    println!("🦁 AfriForme v0.20 — La plateforme africaine de code");
    println!("📡 Serveur: http://localhost:{}", port);
    println!("👤 Utilisateurs: {}", state.lock().unwrap().users.len());
    println!("📦 Depots: {}", state.lock().unwrap().repos.len());
    println!("📖 Le Griot: Actif");
    println!("🎓 Cours: {}", state.lock().unwrap().courses.len());
    println!("💚 Zero dependance — Rust std only");
    println!("---");

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Port en cours");
    println!("✅ AfriForme en ligne sur le port {}", port);

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let state = Arc::clone(&state);
            thread::spawn(move || {
                handle_request(stream, state);
            });
        }
    }
}
