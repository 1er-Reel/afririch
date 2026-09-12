// ===== AFRI QUIZ — La Monnaie Intelligente v1.66 =====
// "L'Afrique sera intelligente à force d'avoir des Afri."
// Les questions sont posées selon TON pays — ta propre culture.
// Bonne réponse = graines (1 graine = 0.00000001 AFR = $0.01).
// v1.66 — construit pour le chef. 💚🦁

/// Une question culturelle — 4 choix, une bonne réponse, une explication
#[derive(Clone, Debug)]
pub struct QuestionQuiz {
    pub question: String,
    pub choix: Vec<String>,
    pub bonne: usize,        // index de la bonne réponse (0-3)
    pub explication: String, // le pourquoi — l'Afrique enseigne
}

/// Graines gagnées par bonne réponse
pub const GRAINES_PAR_BONNE: u64 = 15;

/// Petit constructeur pour écrire les questions proprement
fn q(question: &str, c1: &str, c2: &str, c3: &str, c4: &str, bonne: usize, explication: &str) -> QuestionQuiz {
    QuestionQuiz {
        question: question.to_string(),
        choix: vec![c1.to_string(), c2.to_string(), c3.to_string(), c4.to_string()],
        bonne,
        explication: explication.to_string(),
    }
}

/// La banque de questions d'un pays — selon le pays de l'utilisateur.
/// Si le pays n'a pas encore ses questions, on pose les grandes questions
/// panafricaines — l'Afrique entière est notre culture.
pub fn questions_pays(country: &str) -> Vec<QuestionQuiz> {
    match country {
        "Mali" => vec![
            q("Qui gouvernait l'empire du Mali au 16e siècle ?",
              "Soundiata Keita", "Askia Mohammed", "Mansa Moussa", "Samori Touré", 1,
              "Askia Mohammed a pris le pouvoir en 1493 et fait de l'empire songhaï la grande puissance du Sahel au 16e siècle."),
            q("Qui a fondé l'empire du Mali au 13e siècle après la bataille de Kirina ?",
              "Soundiata Keita", "Sunni Ali", "Kankan Moussa", "Alpha Bakri", 0,
              "Soundiata Keita, le Lion du Mali, a vaincu le roi sosso Soumaoro Kanté vers 1235 à Kirina."),
            q("Quel empereur a distribué tant d'or au Caire que son passage a fait baisser le prix de l'or ?",
              "Aboubacri II", "Mansa Moussa", "Askia Daoud", "Sékou Amadou", 1,
              "Mansa Moussa, en pèlerinage à La Mecque en 1324, a étonné le monde par la richesse du Mali."),
        ],
        "Niger" => vec![
            q("Quelle ville du Niger était le cœur de l'empire Songhaï ?",
              "Agadez", "Gao", "Zinder", "Tahoua", 1,
              "Gao, sur le fleuve Niger, était la capitale de l'empire Songhaï avant de passer au Mali puis de renaître."),
            q("Quel explorateur africain a décrit Tombouctou et Gao au 16e siècle ?",
              "Ibn Battuta", "Léon l'Africain", "Al-Bakri", "Mungo Park", 1,
              "Léon l'Africain (Hassan al-Wazzan) a traversé le Sahel et décrit la richesse des villes du fleuve Niger."),
            q("Dans quel massif du Niger vit le peuple touareg depuis des siècles ?",
              "Le delta intérieur", "L'Aïr", "Le Termit", "L'Adrar des Ifoghas", 1,
              "Le massif de l'Aïr, au nord du Niger, est le pays des Touaregs, les hommes bleus du désert."),
        ],
        "Burkina Faso" => vec![
            q("Quel président a dit « La patrie ou la mort, nous vaincrons » ?",
              "Maurice Yaméogo", "Thomas Sankara", "Blaise Compaoré", "Sangoulé Lamizana", 1,
              "Thomas Sankara, le président panafricain, a transformé la Haute-Volta en Burkina Faso : « le pays des hommes intègres »."),
            q("Comment s'appelait le Burkina Faso avant 1984 ?",
              "Le Mossi", "La Haute-Volta", "Le Wagadou", "Le Gourma", 1,
              "Sankara a renommé la Haute-Volta en Burkina Faso le 4 août 1984."),
            q("Quel peuple a fondé les royaumes Mossi, cœur historique du Burkina ?",
              "Les Peuls", "Les Mossi", "Les Bobo", "Les Gourmantché", 1,
              "Les Mossi ont bâti des royaumes puissants comme le Ouagadougou et le Yatenga, jamais vraiment soumis par les empires voisins."),
        ],
        "Sénégal" => vec![
            q("Quelle héroïne casamançaise a résisté à la colonisation et est surnommée la reine de Kabrousse ?",
              "Aline Sitoe Diatta", "Ndaté Yalla", "Sarraounia", "Yennega", 0,
              "Aline Sitoe Diatta a mené la résistance en Casamance avant d'être déportée — elle est morte loin de sa terre, mais jamais soumise."),
            q("Quel président-poète a défendu la négritude dans le monde entier ?",
              "Léopold Sédar Senghor", "Abdou Diouf", "Macky Sall", "Lamine Guèye", 0,
              "Senghor, poète et premier président du Sénégal, a fait de la négritude une fierté mondiale."),
            q("Quelle ville fut la capitale de l'Afrique Occidentale Française ?",
              "Saint-Louis", "Dakar", "Conakry", "Bamako", 1,
              "Dakar a été la capitale de l'AOF de 1902 à 1960."),
        ],
        "Ghana" => vec![
            q("Le Ghana moderne porte le nom de quel empire médiéval ?",
              "L'empire du Ghana (Wagadou)", "L'empire du Bénin", "Le royaume Kongo", "L'empire Ashanti", 0,
              "L'empire du Ghana (Wagadou) contrôlait l'or et le sel du 8e au 11e siècle — Nkrumah a repris ce nom glorieux en 1957."),
            q("Qui est devenu le premier président du Ghana indépendant ?",
              "Kofi Busia", "Kwame Nkrumah", "Jerry Rawlings", "J.B. Danquah", 1,
              "Kwame Nkrumah a conduit le Ghana à l'indépendance en 1957, la première colonie britannique d'Afrique subsaharienne libre."),
            q("Quelle reine ashanti a mené la guerre de l'Âge d'Or contre les Britanniques en 1900 ?",
              "Yaa Asantewaa", "Nana Yaa", "Efua Sutherland", "Ama Ata Aidoo", 0,
              "Yaa Asantewaa, reine-mère d'Edweso, a dirigé la dernière grande guerre ashanti pour défendre le Âge d'Or royal."),
        ],
        "Nigeria" => vec![
            q("Quel empire du nord du Nigeria fut fondé par Ousmane dan Fodio en 1804 ?",
              "L'empire du Kanem", "Le Califat de Sokoto", "Le royaume du Bénin", "L'empire d'Oyo", 1,
              "Ousmane dan Fodio a fondé le Califat de Sokoto, un des plus grands États africains du 19e siècle."),
            q("Quel royaume est célèbre pour ses bronzes magnifiques pillés en 1897 ?",
              "Le royaume du Bénin", "Le royaume haoussa", "Le royaume Nupe", "Le royaume Igbo", 0,
              "Les bronzes du Bénin sont des chefs-d'œuvre de l'art mondial — le Nigeria exige leur retour."),
            q("Quel Nigérian a été le premier lauréat africain du prix Nobel de littérature ?",
              "Chinua Achebe", "Wole Soyinka", "Chimamanda Adichie", "Fela Kuti", 1,
              "Wole Soyinka a reçu le Nobel de littérature en 1986."),
        ],
        "Côte d'Ivoire" => vec![
            q("Qui fut le premier président de la Côte d'Ivoire indépendante ?",
              "Félix Houphouët-Boigny", "Henri Konan Bédié", "Laurent Gbagbo", "Alassane Ouattara", 0,
              "Houphouët-Boigny a dirigé la Côte d'Ivoire de 1960 à 1993 et fait du pays le « miracle ivoirien »."),
            q("Quelle ville est la capitale politique de la Côte d'Ivoire ?",
              "Abidjan", "Bouaké", "Yamoussoukro", "San-Pédro", 2,
              "Yamoussoukro est la capitale officielle depuis 1983, avec sa basilique parmi les plus grandes du monde."),
            q("Quelle reine a fondé le royaume baoulé selon la légende ?",
              "Aura Poku", "Aline Sitoe", "Pokou", "Yennega", 0,
              "La reine Aura Poku aurait sacrifié son enfant au fleuve pour permettre à son peuple de le traverser — d'où le nom Baoulé."),
        ],
        "Guinée" => vec![
            q("Quel président a dit « Non » à la France en 1958 ?",
              "Ahmed Sékou Touré", "Samori Touré", "Alpha Condé", "Lansana Conté", 0,
              "Sékou Touré : « Nous préférons la liberté dans la pauvreté à la richesse dans l'esclavage. » La Guinée a voté Non à de Gaulle."),
            q("Qui a fondé l'empire Wassoulou et résisté des années aux armées françaises ?",
              "Samori Touré", "Bocar Biro", "Alpha Yaya", "Din Salifou", 0,
              "Samori Touré, l'Almamy, a créé un empire et une armée moderne avant d'être capturé en 1898."),
            q("Quelle femme a milité pour l'indépendance de la Guinée aux côtés de Sékou Touré ?",
              "Mafory Bangoura", "Aissatou Bah", "Kaba Kadiatou", "Nènè Fatou", 0,
              "Mafory Bangoura, figure du RDA, a lutté pour l'indépendance guinéenne."),
        ],
        "Bénin" => vec![
            q("Les « Amazones », femmes soldats redoutées du 19e siècle, venaient de quel royaume ?",
              "Le royaume du Dahomey", "Le royaume d'Oyo", "L'empire du Mali", "Le royaume Kongo", 0,
              "Les Agojié du Dahomey étaient un corps d'élite de femmes soldats, craint par tous les envahisseurs."),
            q("Comment s'appelait le Bénin avant 1975 ?",
              "Le Dahomey", "Le Biafra", "Le Wagadou", "L'Atakora", 0,
              "Le Dahomey est devenu le Bénin en 1975, en hommage à la civilisation du golfe de Guinée."),
            q("Quelle ville béninoise abrite la cité lacustre de Ganvié ?",
              "Cotonou", "Porto-Novo", "Abomey-Calavi", "Ouidah", 2,
              "Ganvié, près d'Abomey-Calavi sur le lac Nokoué, est la « Venise de l'Afrique » — village bâti sur l'eau."),
        ],
        "Togo" => vec![
            q("Quelle est la capitale du Togo ?",
              "Sokodé", "Kara", "Lomé", "Atakpamé", 2,
              "Lomé, sur le golfe de Guinée, est la capitale du Togo."),
            q("Quel peuple du Togo et du Ghana est célèbre pour ses sculptures sur bois et ses poids à peser l'or ?",
              "Les Ewe", "Les Kabyè", "Les Mina", "Les Bassar", 0,
              "Les Ewe sont réputés pour leur art, leurs tissus kente et leur riche tradition orale."),
            q("Le Togo a obtenu son indépendance de quel pays en 1960 ?",
              "La France", "Le Royaume-Uni", "L'Allemagne", "La Belgique", 0,
              "Le Togo, ancienne colonie allemande puis sous mandat français, est devenu indépendant de la France le 27 avril 1960."),
        ],
        "Cameroun" => vec![
            q("Quelle ville fut la capitale de l'Afrique Équatoriale Française puis du Cameroun allemand ?",
              "Douala", "Yaoundé", "Garoua", "Bafoussam", 0,
              "Douala, première ville économique, fut le point d'entrée de la colonisation avant que Yaoundé ne devienne la capitale politique."),
            q("Quel roi du Cameroun a résisté à la colonisation allemande et fut exilé ?",
              "Rudolf Douala Manga Bell", "Njoya", "Bello", "Ateba", 0,
              "Le roi Rudolf Douala Manga Bell a été pendu par les Allemands en 1914 pour avoir défendu son peuple."),
            q("Quel footballeur camerounais a étonné le monde à la Coupe du Monde 1990 ?",
              "Roger Milla", "Samuel Eto'o", "Thomas Nkono", "Abedi Pelé", 0,
              "Roger Milla, à 38 ans, a fait danser le Cameroun jusqu'aux quarts de finale — l'Afrique entière a vibré."),
        ],
        "Congo" => vec![
            q("Quel royaume africain était célèbre pour son art et sa chrétienté kongo au 15e siècle ?",
              "Le royaume Kongo", "L'empire Luba", "Le royaume Loango", "L'empire Lunda", 0,
              "Le royaume Kongo, avec sa capitale Mbanza Kongo, commerçait avec le Portugal dès 1483."),
            q("Quelle capitale congolaise porte le nom d'un explorateur italien au service du Portugal ?",
              "Brazzaville", "Pointe-Noire", "Djambala", "Owando", 0,
              "Brazzaville honore Pierre Savorgnan de Brazza — face à Kinshasa, les deux capitales se regardent sur le fleuve."),
            q("Quel président congolais a incarné la lutte contre le colonialisme dans les années 1960 ?",
              "Fulbert Youlou", "Alphonse Massamba-Débat", "Marien Ngouabi", "Denis Sassou Nguesso", 1,
              "Alphonse Massamba-Débat a mené le Congo vers la révolution et le socialisme africain."),
        ],
        "RD Congo" => vec![
            q("Qui fut le premier Premier ministre de la RD Congo indépendante en 1960 ?",
              "Patrice Lumumba", "Mobutu Sese Seko", "Joseph Kasa-Vubu", "Moïse Tshombe", 0,
              "Patrice Lumumba, héros de l'indépendance congolaise, a été assassiné en 1961 — l'Afrique ne l'oublie pas."),
            q("Quel royaume du Katanga résista à la colonisation belge ?",
              "Le royaume Yeke", "Le royaume Luba", "Le royaume Kuba", "Le royaume Lunda", 0,
              "Msiri, roi du Yeke (Garanganze), a tenu le Katanga jusqu'à sa mort face aux Belges en 1891."),
            q("Quelle musique née à Kinshasa fait danser toute l'Afrique ?",
              "La rumba congolaise", "Le highlife", "Le mbalax", "Le taarab", 0,
              "La rumba congolaise, inscrite au patrimoine de l'UNESCO, est l'âme de Kinshasa."),
        ],
        "Chad" | "Tchad" => vec![
            q("Quel empereur du Kanem-Bornou dominait le Sahel autour du lac Tchad ?",
              "Idris Aloma", "Sunni Ali", "Askia Mohammed", "Mai Dunama", 0,
              "Idris Aloma (16e siècle) a fait du Bornou une grande puissance du Sahel."),
            q("Quelle femme touarègue est devenue une légende de la résistance au Tchad ?",
              "Kaltsoumia", "Tania", "Aïssata", "Fatimé", 0,
              "Les femmes touarègues ont toujours tenu une place centrale — la ténacité du Tchad est leur héritage."),
            q("Le Tchad tire son nom de quel lac ?",
              "Le lac Tchad", "Le lac Tanganyika", "Le lac Victoria", "Le lac Nyassa", 0,
              "Le lac Tchad, cœur d'eau du Sahel, a nourri les empires Kanem-Bornou depuis mille ans."),
        ],
        _ => vec![
            // Grandes questions panafricaines — pour tous les autres pays
            q("Qui a fondé l'Organisation de l'Unité Africaine en 1963 ?",
              "Kwame Nkrumah et les chefs d'État africains", "Nelson Mandela", "Muammar Kadhafi", "Léopold Senghor", 0,
              "L'OUA, née à Addis-Abeba en 1963, est devenue l'Union Africaine — le rêve de l'unité continue."),
            q("Quel est le plus grand désert du monde, qui couvre le nord de l'Afrique ?",
              "Le Sahara", "Le Kalahari", "Le Gobi", "Le Namib", 0,
              "Le Sahara, 9 millions de km², était autrefois vert — ses fresques de Tassili racontent cette Afrique luxuriante."),
            q("Quel fleuve est le plus long d'Afrique ?",
              "Le Nil", "Le Congo", "Le Niger", "Le Zambèze", 0,
              "Le Nil, 6 650 km, a nourri l'Égypte pharaonique — il naît dans les lacs d'Afrique de l'Est."),
            q("Quel royaume d'Afrique australe a bâti la grande muraille de Grand Zimbabwe ?",
              "Le royaume de Zimbabwe", "Le royaume Mutapa", "Le royaume Zoulou", "Le royaume Kongo", 0,
              "Grand Zimbabwe, bâtie de pierre sans mortier au 11e siècle, prouve la maîtrise architecturale africaine."),
            q("Qui est l'auteur de « Things Fall Apart », le roman africain le plus lu au monde ?",
              "Chinua Achebe", "Wole Soyinka", "Ngugi wa Thiongo", "Ayi Kwei Armah", 0,
              "Chinua Achebe a raconté l'Igbo du Nigeria au monde entier."),
        ],
    }
}
