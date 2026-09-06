// AfriForme v0.5 — La plateforme africaine de code
// Comme GitHub + Copilot, mais souverain, africain, zero dependance
// Par Koffi Christ Olivier & Letta-Chan
// Rust std only — Cargo.toml [dependencies] vide
// v0.2: Cours auto-generees + Exercices + Diplomes pour chaque depot
// v0.3: Profils utilisateurs + Catalogue de cours + Stars
// v0.4: Classement + README + Recherche
// v0.5: Fil d activite + Fork + Commentaires

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
    is_public: bool,
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

struct AppState {
    users: Vec<User>,
    repos: Vec<Repository>,
    sessions: HashMap<String, String>, // session_token -> username
    ai_chats: HashMap<String, AIChat>, // username -> chat history
    courses: Vec<Course>, // auto-generated courses for repos
    starred: HashMap<String, Vec<usize>>, // username -> repo IDs starred
    comments: Vec<Comment>, // comments on repos
    next_repo_id: usize,
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
            next_repo_id: 1,
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
            repos_json.push_str(&format!(
                r#"{{"id":{},"owner":"{}","name":"{}","description":"{}","language":"{}","stars":{},"forks":{},"created_at":"{}","files":{},"is_public":{}}}"#,
                r.id,
                escape_json(&r.owner),
                escape_json(&r.name),
                escape_json(&r.description),
                escape_json(&r.language),
                r.stars,
                r.forks,
                escape_json(&r.created_at),
                files_json,
                r.is_public
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
            starred_json.push_str(&format!(r#""{}":"[{}]""#, escape_json(k), ids.join(",")));
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

    fn fork_repo(&mut self, owner: &str, repo_name: &str, new_owner: &str) -> Option<usize> {
        // Clone repo data to avoid borrow conflict
        let repo_data = self.find_repo(owner, repo_name).map(|r| {
            (r.description.clone(), r.language.clone(), r.files.clone())
        });
        if let Some((desc, lang, files)) = repo_data {
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
                description: format!("Fork de {}/{} — {}", owner, repo_name, desc),
                language: lang,
                stars: 0,
                forks: 0,
                created_at: now_string(),
                files,
                is_public: true,
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
                    is_public,
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
            content: "Un serveur HTTP ecoute les requetes des clients (navigateurs) et repond avec des pages web. Le serveur tourne sur un port (ex: 8090) et attend les connexions.\n\nCe projet contient un serveur HTTP — il fait partie de l'infrastructure internet africaine souveraine.".to_string(),
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
            explanation: "AfriForme tourne sur le port 8090. Le serveur HTTP ecoute les requetes sur ce port et repond avec des pages HTML.".to_string(),
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
// AI ASSISTANT (Copilot-like)
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
    format!("Je comprends ta question: \"{}\". Je suis encore en developpement, mais j'apprends. Essaie de me demander sur Rust, Python, blockchain, Termux, Git, ou AfriChain. Je suis ton Copilot africain.", question)
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
<a href="/explore">Explorer</a>
<a href="/courses">🎓 Cours</a>
<a href="/leaderboard">🏆 Classement</a>
<a href="/search">🔍 Rechercher</a>
<a href="/explore">📦 Explorer</a>
<a href="/ai">🤖 IA Copilot</a>
<a href="/register">S'inscrire</a>
<a href="/login">Connexion</a>
</div>
</nav>
<div class="container">
{}
</div>
<div class="footer">🦁 AfriForme v0.5 — La plateforme africaine de code — Par Koffi Christ Olivier & Letta-Chan — Rust std only, zero dependance</div>
</body>
</html>"##, title, body)
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
                r.stars, r.forks, r.created_at
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
                r.owner, r.name, r.owner, r.name, r.created_at
            ));
        }
        // Recent comments (last 5)
        let mut recent_comments: Vec<&Comment> = state.comments.iter().collect();
        recent_comments.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        for c in recent_comments.iter().take(5) {
            if let Some(repo) = state.repos.iter().find(|r| r.id == c.repo_id) {
                activities.push(format!(
                    r#"<div class="card" style="padding:10px;margin:5px 0;"><span style="color:#f59e0b;">💬 Commentaire</span> — <strong>{}</strong> sur <a href="/{}/{}">{}/{}</a> <span style="color:#8b949e;">· {}</span></div>"#,
                    c.author, repo.owner, repo.name, repo.owner, repo.name, c.created_at
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
                r.language, r.description, r.stars, r.created_at
            )).collect::<Vec<_>>().join("")
        };
        format!(r#"
<div style="text-align:center;padding:30px 0;">
<h1>🦁 AfriForme</h1>
<p style="font-size:1.1em;color:#8b949e;">La plateforme africaine de code — souveraine, zero dependance</p>
<p style="margin:15px 0;">Comme GitHub + Copilot, mais africain. Inscription gratuite.</p>
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
<p style="color:#8b949e;margin:8px 0;">Tu es developpeur? Rejoins la communaute africaine du code. Cree tes depots, partage ton code, apprends avec l'IA Copilot.</p>
<div style="display:flex;gap:10px;flex-wrap:wrap;margin-top:10px;">
<a href="/register" class="btn">🚀 S'inscrire comme developpeur</a>
<a href="/explore" class="btn btn-secondary">🔍 Explorer les depots</a>
<a href="/ai" class="btn btn-secondary">🤖 IA Copilot</a>
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
<button type="submit">Creer le depot</button>
</form>
</div>
"#;
    html_page("Nouveau depot", body)
}

fn html_repo_view(repo: &Repository, owner: &str, name: &str, is_owner: bool, current_user_opt: Option<&str>, comments_html: &str) -> String {
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

    let owner_actions = if is_owner {
        format!(r#"<a href="/{}/{}/upload" class="btn btn-secondary">+ Ajouter fichier</a>"#, owner, name)
    } else {
        String::new()
    };

    let star_form = if let Some(_user) = current_user_opt {
        format!(r#"<form method="POST" action="/{}/{}/star" style="display:inline;"><button type="submit" class="btn btn-secondary">⭐ Star</button></form>"#, owner, name)
    } else {
        String::new()
    };

    let fork_form = if let Some(user) = current_user_opt {
        if user != owner {
            format!(r#"<form method="POST" action="/{}/{}/fork" style="display:inline;"><button type="submit" class="btn btn-secondary">🍴 Fork</button></form>"#, owner, name)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Comments section
    let comments_html = {
        // We need the repo_id to get comments, but we only have repo here
        // Since repo is borrowed, we can use repo.id directly
        format!("<h2>💬 Commentaires</h2>")
    };

    let body = format!(r#"
<div style="display:flex;justify-content:space-between;align-items:center;">
<div>
<h1>{}/<span style="color:#58a6ff;">{}</span></h1>
<p style="color:#8b949e;">{}</p>
</div>
<div>
<span class="badge badge-{}">{}</span>
{} <span class="badge {}">{}</span> {} {}
</div>
</div>
<div class="stats">
<div class="stat"><div class="num">⭐ {}</div><div class="label">Stars</div></div>
<div class="stat"><div class="num">🍴 {}</div><div class="label">Forks</div></div>
<div class="stat"><div class="num">{}</div><div class="label">Fichiers</div></div>
</div>
<div style="margin:10px 0;">
<a href="/course/{}/{}" class="btn" style="background:#f59e0b;">🎓 Cours & Diplome</a>
</div>
{}
{}
{}
"#, owner, name, repo.description, owner, name,
    if repo.language == "Rust" { "rust" } else if repo.language == "Python" { "python" } else { "js" },
    repo.language,
    owner_actions,
    if repo.is_public { "badge-public" } else { "badge-private" },
    if repo.is_public { "Public" } else { "Prive" },
    star_form, fork_form,
    repo.stars, repo.forks, repo.files.len(),
    readme_html, files_html, comments_html
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
<textarea name="content" placeholder="Contenu du fichier" rows="15" required></textarea>
<button type="submit">Ajouter le fichier</button>
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
            r.language, r.description, r.stars, r.created_at
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

    let total_stars: usize = user_repos.iter().map(|r| r.stars).sum();

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
<div class="stat"><div class="num">⭐ {}</div><div class="label">Stars recues</div></div>
</div>
</div>
<h2>📦 Depots</h2>
{}
<h2>🎓 Diplomes</h2>
{}
"#, user.username, user.country, user.created_at,
    if user.bio.is_empty() { "Pas de bio encore." } else { &user.bio },
    edit_bio,
    user_repos.len(), diplomas.len(), total_stars,
    repos_html, diplomas_html);

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
                <div class="stat"><div class="num">⭐ {}</div><div class="label">Stars</div></div>
                </div>
                </div>"#,
                medal, username, username, country, repos, diplomas, stars
            )
        }).collect::<Vec<_>>().join("")
    };

    let body = format!(r#"
<h1>🏆 Classement des Developpeurs</h1>
<p style="color:#8b949e;">Les meilleurs developpeurs africains — classe par diplomes, stars et depots</p>
{}
"#, leaderboard_html);

    html_page("Classement", &body)
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
            r.language, r.description, r.stars, r.created_at
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

fn html_explore(state: &AppState) -> String {
    let public_repos: Vec<&Repository> = state.repos.iter().filter(|r| r.is_public).collect();
    let repos_html = if public_repos.is_empty() {
        "<div class='empty'>Aucun depot public pour le moment.</div>".to_string()
    } else {
        public_repos.iter().map(|r| format!(
            r#"<div class="repo"><h3><a href="/{}/{}">{}/{}</a> <span class="badge badge-{}">{}</span></h3><div class="desc">{}</div><div class="meta">⭐ {} · 🍴 {} · {}</div></div>"#,
            r.owner, r.name, r.owner, r.name,
            if r.language == "Rust" { "rust" } else if r.language == "Python" { "python" } else { "js" },
            r.language, r.description, r.stars, r.forks, r.created_at
        )).collect::<Vec<_>>().join("")
    };

    let body = format!(r#"
<h1>Explorer les depots</h1>
<p style="color:#8b949e;">Decouvrez les projets de la communaute africaine</p>
{}
"#, repos_html);

    html_page("Explorer", &body)
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
<h1>🤖 IA Copilot Africain</h1>
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

    html_page("IA Copilot", &body)
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
                        is_public,
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
        ("GET", "/search") => {
            let s = state.lock().unwrap();
            let q = query_params.get("q").cloned().unwrap_or_default();
            ("200", "text/html; charset=utf-8", html_search(&s, current_user.as_deref(), &q))
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

                let s = state.lock().unwrap();
                if let Some(repo) = s.find_repo(owner, repo_name) {
                    let is_owner = current_user.as_deref() == Some(owner);
                    if sub.starts_with("file/") {
                        let filename = &sub[5..];
                        if let Some(content) = repo.files.get(filename) {
                            ("200", "text/html; charset=utf-8", html_file_view(repo, owner, repo_name, filename, content, is_owner))
                        } else {
                            ("404", "text/html; charset=utf-8", html_page("404", "<div class='empty'>Fichier non trouve</div>"))
                        }
                    } else if sub == "upload" && is_owner {
                        ("200", "text/html; charset=utf-8", html_upload(owner, repo_name))
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
                                    c.author, c.created_at, c.text
                                ));
                            }
                            if current_user.is_some() {
                                html.push_str(&format!(r#"<form method="POST" action="/{}/{}/comment" style="margin:10px 0;"><input type="text" name="text" placeholder="Ecrire un commentaire..." style="width:70%;"><button type="submit">Envoyer</button></form>"#, owner, repo_name));
                            }
                            html
                        };
                        drop(s);
                        let s2 = state.lock().unwrap();
                        let repo2 = s2.find_repo(owner, repo_name).unwrap();
                        ("200", "text/html; charset=utf-8", html_repo_view(repo2, owner, repo_name, is_owner, current_user.as_deref(), &comments_html))
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
                            s.toggle_star(user, repo_id);
                            s.save();
                        }
                        ("302", "text/html", format!("Location: /{}/{}", owner, repo_name))
                    } else {
                        ("302", "text/html", "Location: /login".to_string())
                    }
                } else if action == "fork" {
                    if let Some(user) = &current_user {
                        let mut s = state.lock().unwrap();
                        s.fork_repo(owner, repo_name, user);
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
                        if let Some(repo) = s.find_repo_mut(owner, repo_name) {
                            repo.files.insert(filename, content);
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
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
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
    let port = 8090;
    let state = Arc::new(Mutex::new(AppState::new()));

    println!("🦁 AfriForme v0.5 — La plateforme africaine de code");
    println!("📡 Serveur: http://localhost:{}", port);
    println!("👤 Utilisateurs: {}", state.lock().unwrap().users.len());
    println!("📦 Depots: {}", state.lock().unwrap().repos.len());
    println!("🤖 IA Copilot: Active");
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
