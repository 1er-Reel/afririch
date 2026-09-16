// ============================================================
// v1.78 — LA MÉMOIRE DU CONTINENT 🌍🧠
// SAHARA AFRI, notre Google souverain : taper « Mali » donne
// l'HISTOIRE VRAIE du Mali — empires, ethnies, langues, passé,
// présent, futur. Chaque langue africaine a son AI Autonome,
// nourrie par les jeunes qui collectent les mots des anciens
// dans les villages. Chaque mot est gravé sur la blockchain.
// Zéro dépendance externe — Rust std uniquement. 💚🦁
// ============================================================

use crate::afri_json::{JsonValue, from_str, to_string};
use crate::afri_time::now_timestamp;
use std::collections::HashMap;

// ===== LA FICHE PAYS — la vraie histoire de l'Afrique =====

pub struct FichePays {
    pub nom: &'static str,
    pub drapeau: &'static str,
    pub capitale: &'static str,
    pub ethnies: Vec<&'static str>,
    pub langues: Vec<&'static str>,
    pub histoire: Vec<&'static str>,
    pub present: &'static str,
    pub futur: &'static str,
}

// Les 14 pays fondateurs — fiches complètes, histoire vraie.
pub fn fiches_pays() -> Vec<FichePays> {
    vec![
        FichePays {
            nom: "Mali", drapeau: "🇲🇱", capitale: "Bamako",
            ethnies: vec!["Bambara", "Malinké", "Peul", "Soninké", "Songhaï", "Touareg (Kel Tamasheq)", "Dogon", "Bozo", "Khassonké", "Sénoufo", "Minianka"],
            langues: vec!["Bambara", "Peul (Fulfulde)", "Soninké", "Songhaï", "Tamasheq", "Dogon (Dogoso)", "Malinké", "Bozo", "Khassonké"],
            histoire: vec![
                "Avant tout : l'Empire du Ghana (Wagadu), né au Mali et en Mauritanie — les Soninké bâtirent la première richesse de l'Afrique de l'Ouest par le sel et l'or.",
                "1235 — Soundiata Keïta, le Prince Lion, vainc Soumaoro Kanté à la bataille de Kirina. Naissance de l'Empire du Mali, l'un des plus vastes du monde. La Charte de Kurukan Fuga, une des premières constitutions de l'humanité, proclame les droits et les devoirs.",
                "1324 — Mansa Moussa, l'homme le plus riche de l'histoire, part en pèlerinage à La Mecque avec 60 000 personnes et tellement d'or que le cours de l'or s'effondre au Caire.",
                "Tombouctou, Gao, Djenné : des universités comme Sankoré attirent les savants du monde entier. Les manuscrits de Tombouctou — des centaines de milliers — prouvent que l'Afrique écrivait les mathématiques, l'astronomie et le droit.",
                "XVe siècle — l'Empire Songhaï prend le flambeau : Sonni Ali Ber, puis Askia Mohammed le Grand font de Gao la capitale d'un empire plus grand que l'Europe occidentale.",
                "1892 — la colonisation française détruit les royaumes. Le Soudan français est coupé de ses routes commerciales millénaires.",
                "22 septembre 1960 — indépendance. Modibo Keïta, le président-socialiste, crée le franc malien et résiste à la dépendance.",
                "2012-2024 — épreuves : crise au Nord, rebellions Touareg noyées dans le djihadisme, interventions étrangères. Puis le Mali se relève : sortie des accords de défense avec la France, alliance avec le Niger et le Burkina Faso.",
                "2024 — le Mali, le Niger et le Burkina fondent l'AES (Alliance des États du Sahel), la première confédération souveraine d'Afrique de l'Ouest depuis les empires.",
            ],
            present: "Le Mali reprend sa souveraineté : AES, monnaie en projet, armée reconstituée, retour aux partenaires qui respectent l'Afrique. La jeunesse de Bamako à Tombouctou porte l'héritage de Soundiata.",
            futur: "Un Mali où Tombouctou redevient une capitale du savoir mondial, où le fleuve Niger porte le commerce africain, où les 4e et 5e empires sont numériques et construits par les jeunes sur AfriChain.",
        },
        FichePays {
            nom: "Niger", drapeau: "🇳🇪", capitale: "Niamey",
            ethnies: vec!["Haoussa", "Zarma-Songhaï", "Peul (Fulani)", "Kanouri", "Touareg (Kel Tamasheq)", "Toubou", "Gurma"],
            langues: vec!["Haoussa", "Zarma", "Peul (Fulfulde)", "Kanouri", "Tamasheq", "Toubou", "Gurma"],
            histoire: vec![
                "Le berceau des empires : le Kanem-Bornou, un des royaumes les plus anciens d'Afrique (VIIIe siècle), régnait autour du lac Tchad avec les Kanouri.",
                "Le Sultanat d'Agadez, fondé en 1449, devint le carrefour des caravanes transsahariennes — l'or, le sel, les manuscrits. La mosquée d'Agadez, en banco, est la plus haute du monde en terre.",
                "Les Touareg du Aïr et du Azawagh tinrent les routes du désert pendant mille ans — le Tamasheq, leur langue, s'écrit en tifinagh depuis 3000 ans.",
                "1922 — colonisation française : le Niger devient une colonie coupée de ses routes du nord.",
                "3 août 1960 — indépendance avec Hamani Diori. Puis Seyni Kountché, puis la route des alternances.",
                "2023 — le peuple refuse le néocolonialisme : le général Tchiani prend la direction, le Niger rejoint l'AES avec le Mali et le Burkina.",
            ],
            present: "Le Niger a l'uranium qui alimente les lumières de Paris, l'or de Samira, le pétrole d'Agadem — et décide enfin QUI en profite. Membre fondateur de l'AES.",
            futur: "L'uranium, l'or et le soleil du Sahara alimenteront l'industrie africaine, pas seulement l'Europe. Agadez redevient la porte du savoir saharien.",
        },
        FichePays {
            nom: "Burkina Faso", drapeau: "🇧🇫", capitale: "Ouagadougou",
            ethnies: vec!["Mossi", "Peul", "Bissa", "Bobo", "Gourmantché", "Dagara", "Lobi", "Sénoufo", "Samo"],
            langues: vec!["Mooré", "Dioula", "Peul (Fulfulde)", "Bissa", "Bobo", "Gourmantché", "Dagara", "San"],
            histoire: vec![
                "XVe siècle — la princesse Yennenga, la mère du peuple Mossi, fuit le royaume de son père sur sa jument. Son fils Ouedraogo donne naissance aux royaumes Mossi : Tenkodogo, Ouagadougou, Yatenga — des royaumes qui résistèrent aux empires et aux colonisateurs.",
                "1896 — colonisation française : la Haute-Volta est coupée de l'Ashanti et du Soudan.",
                "5 août 1960 — indépendance avec Maurice Yaméogo.",
                "4 août 1983 — Thomas Sankara, le président de l'intégrité, renverse l'ordre : « La patrie ou la mort, nous vaincrons ». En 4 ans : vaccination de 2,5 millions d'enfants, plantage de 10 millions d'arbres, logements Faso, femmes ministres, refus de la dette. Il disait : « Qui ne feed pas sa famille est un esclave. »",
                "15 octobre 1987 — Sankara est assassiné. Blaise Compaoré dirige 27 ans.",
                "2014 — le peuple se lève (l'insurrection d'octobre) et chasse Compaoré.",
                "2022 — le capitaine Ibrahim Traoré prend la direction. Le Burkina rejoint l'AES, Sankara est réhabilité, sa statue veille sur Ouaga.",
            ],
            present: "Le pays des Hommes intègres (Burkina Faso = « terre des hommes debout ») mène la souveraineté sous le capitaine Ibrahim Traoré — le plus jeune président du monde, symbole de la génération AfriChain.",
            futur: "Le Burkina nourrira l'AES : coton transformé sur place, or burkinabè pour l'industrie burkinabè, et la jeunesse de Bobo-Dioulasso bâtira la tech sahelienne.",
        },
        FichePays {
            nom: "Sénégal", drapeau: "🇸🇳", capitale: "Dakar",
            ethnies: vec!["Wolof", "Peul", "Sérère", "Diola", "Mandinka", "Soninké", "Lébou"],
            langues: vec!["Wolof", "Peul (Fulfulde)", "Sérère", "Diola", "Mandinka", "Soninké"],
            histoire: vec![
                "Le Tekrur (IXe siècle), premier royaume islamisé d'Afrique de l'Ouest, naquit sur le fleuve Sénégal — les Soninké et les Peul du Fouta y croisèrent les caravanes.",
                "Les royaumes du Cayor, du Baol, du Waalo et du Sine tinrent tête aux royaumes européens pendant 400 ans. Lat-Dior, le damel du Cayor, mourut au combat contre les Français en 1886 — « Le Cayor ne se vend pas ».",
                "Aline Sitoe Diatta, la messagère de Casamance, résista à la conscription coloniale en 1943 et mourut en déportation — la mère de la résistance sénégalaise.",
                "1960 — indépendance avec Léopold Sédar Senghor, poète-président, chantre de la négritude avec Aimé Césaire.",
                "Le Sénégal n'a JAMAIS connu de coup d'État — la démocratie la plus stable d'Afrique de l'Ouest.",
                "2024 — Bassirou Diomaye Faye, élu à 44 ans, marque le retour du panafricanisme au pouvoir.",
            ],
            present: "Dakar reste le carrefour culturel : Youssou N'Dour, la mosquée de la Divinité, le rallye, les Lions de la Teranga. Gorée rappelle la traite — et la survie.",
            futur: "Le Sénégal, porte océane de l'AES et de la ZLECAf, deviendra la plaque tournante du commerce et de la culture africaine sur l'Atlantique.",
        },
        FichePays {
            nom: "Ghana", drapeau: "🇬🇭", capitale: "Accra",
            ethnies: vec!["Akan", "Ewe", "Ga", "Dagomba", "Guan", "Dagara", "Nzema"],
            langues: vec!["Twi (Akan)", "Ewe", "Ga", "Dagbani", "Dagaare", "Nzema", "Hausa"],
            histoire: vec![
                "L'Empire Ashanti (Asanteman) : Osei Tutu et le prêtre Okomfo Anokye unifièrent les clans par le Tabouret d'Or (Sika Dwa Kofi) — l'âme du peuple Akan. Kumasi devint une capitale de richesse et de diplomatie.",
                "Les Ashanti vainquirent les Britanniques en 1826 et résistèrent cent ans — les guerres Anglo-Ashanti sont des guerres de géants.",
                "1900 — Yaa Asantewaa, reine-mère d'Ejisu, dirige la dernière guerre d'indépendance Ashanti : « Si les hommes ont peur, les femmes se battront. »",
                "6 mars 1957 — Kwame Nkrumah proclame l'indépendance du Ghana : PREMIÈRE colonie d'Afrique subsaharienne libre. « Seek ye first the political kingdom. » Le Ghana porte le nom de l'empire ancien pour honorer l'histoire.",
                "Nkrumah fonde l'OUA en 1963 — le père du panafricanisme moderne.",
            ],
            present: "Le Ghana est la démocratie de référence anglophone : alternances pacifiques, l'or du Ashanti, le port de Tema, la tech d'Accra (le « Silicon Lagoon »).",
            futur: "Le Ghana reliera l'héritage de Nkrumah à la tech : AfriChain Accra, la bourse du cacao africain, et l'unité Akan-Ewe-Ga comme modèle.",
        },
        FichePays {
            nom: "Nigeria", drapeau: "🇳🇬", capitale: "Abuja",
            ethnies: vec!["Haoussa", "Yoruba", "Igbo", "Peul (Fulani)", "Tiv", "Kanouri", "Ijaw", "Igala", "Nupe"],
            langues: vec!["Haoussa", "Yoruba", "Igbo", "Fulfulde", "Tiv", "Kanuri", "Ijaw", "Igala", "Nupe", "Pidgin nigérian"],
            histoire: vec![
                "La civilisation Nok (1500 av. J.-C.) sculptait la terre cuite avant Rome — les plus anciennes sculptures d'Afrique de l'Ouest.",
                "Ife, la cité sacrée Yoruba : Oduduwa descendit avec la chaîne du ciel. Les Yoruba bâtirent Ife, Oyo, Benin — l'art d'Ife étonna le monde (têtes en bronze, XIIIe siècle).",
                "1804 — Ousmane dan Fodio fonde le Califat de Sokoto, un des plus grands états d'Afrique — les Peul unifièrent le Nord.",
                "Le royaume du Benin (Edo) : ses murailles (Iya) furent les plus longues constructions de terre de l'histoire humaine, 4 fois plus longues que la Muraille de Chine.",
                "1914 — les Britanniques amalgament le Nord et le Sud en une colonie : « Nigeria », nom inventé par une journaliste anglaise.",
                "1er octobre 1960 — indépendance. 1967-1970 — la guerre du Biafra, la blessure. Puis le pétrole du Delta, les coups, et le retour de la démocratie en 1999.",
                "Nollywood : le 2e plus grand cinéma du monde par volume. La Afrobeats conquiert la planète.",
            ],
            present: "Le géant de l'Afrique : 220 millions d'habitants, l'économie n°1 du continent, Lagos la mégapole qui ne dort jamais, la tech (Flutterwave, Paystack).",
            futur: "Le Nigeria peut nourrir l'Afrique : la jeunesse de Lagos et Kano reliée par AfriChain fera de l'Afrique de l'Ouest la 3e économie tech mondiale.",
        },
        FichePays {
            nom: "Côte d'Ivoire", drapeau: "🇨🇮", capitale: "Yamoussoukro",
            ethnies: vec!["Baoulé", "Bété", "Sénoufo", "Dioula (Malinké)", "Attié", "Agni", "Yacouba", "Ebrié", "Abouré"],
            langues: vec!["Dioula", "Baoulé", "Bété", "Sénoufo", "Attié", "Agni", "Yacouba"],
            histoire: vec![
                "Les royaumes Bété et Gouro, la reine Ayaoua du Sanwi, les Sénoufo du Nord — des sociétés de la kola et du fer.",
                "Samori Touré, l'Almamy du Wassoulou, résista 16 ans à la France avec son armée organisée et ses forgerons fabriques d'armes. Capturé en 1898, il mourut en exil — le Napoléon africain.",
                "1893 — colonisation française. La Côte d'Ivoire devient le paradis du café et du cacao — plantés par la sueur des paysans.",
                "7 août 1960 — indépendance avec Félix Houphouët-Boigny, 33 ans de règne, la stabilité et le « miracle ivoirien ».",
                "2002-2011 — la crise : rébellion du Nord, élection de 2010 contestée, Laurent Gbagbo transféré à la CPI, Alassane Ouattara réconcilie peu à peu.",
            ],
            present: "La Côte d'Ivoire produit 40% du cacao mondial — mais le chocolat se fabrique ailleurs. La jeunesse d'Abidjan (le bassin ébrié, les maquis, le coupé-décalé) exige la transformation locale.",
            futur: "Le cacao transformé à San-Pédro, l'hévéa transformé à Abidjan : la Côte d'Ivoire devient la Chine agro-industrielle de l'Afrique de l'Ouest.",
        },
        FichePays {
            nom: "Guinée", drapeau: "🇬🇳", capitale: "Conakry",
            ethnies: vec!["Peul (Fulani)", "Malinké", "Soussou", "Kissi", "Toma", "Kpelle"],
            langues: vec!["Peul (Pular)", "Malinké", "Soussou", "Kissi", "Toma", "Kpelle"],
            histoire: vec![
                "Le Fouta Djallon : les Peul y bâtirent une théocratie lettrée (XVIIIe siècle) — l'eau du Niger, du Sénégal et de la Gambie naît dans ses montagnes. Le château de l'eau de l'Afrique de l'Ouest.",
                "Samori Touré naquit en Guinée : le résistant du Wassoulou.",
                "1958 — Ahmed Sékou Touré dit NON à de Gaulle : « Nous préférons la liberté dans la pauvreté à la richesse dans l'esclavage. » La Guinée vote NON au référendum — la SEULE colonie française à refuser la Communauté française. De Gaulle coupa tout, Sékou Touré tint debout.",
                "1984 — Lansana Conté, 24 ans de pouvoir. 2010 — Alpha Condé, premier président élu démocratiquement.",
                "2021 — Mamadi Doumbouya : « La Guinée est libre » — le pays reprend le contrôle de la bauxite (Simandou, le plus grand gisement de fer du monde).",
            ],
            present: "La bauxite guinéenne fabrique l'aluminium du monde entier. Conakry chante : Mory Kanté, la kora, les griots du Fouta.",
            futur: "Simandou + le Fouta Djallon : la Guinée devient la centrale minière et hydraulique de l'Afrique de l'Ouest — l'aluminium africain pour les avions africains.",
        },
        FichePays {
            nom: "Bénin", drapeau: "🇧🇯", capitale: "Porto-Novo",
            ethnies: vec!["Fon", "Yoruba", "Adja", "Bariba", "Dendi", "Mina", "Watari"],
            langues: vec!["Fon", "Yoruba", "Adja", "Bariba", "Dendi", "Mina"],
            histoire: vec![
                "Le Danxomè (Dahomey) : le royaume d'Abomey, avec ses Amazones (Agojié) — le seul corps d'armée féminin de l'histoire. Les Agojié terrifiaient les envahisseurs : « Elles se battaient comme des lionnes. »",
                "Ouidah : la route des esclaves, le mémorial de la porte du Non-Retour. Le Bénin porte la mémoire de la traite pour toute l'Afrique.",
                "1894 — colonisation : le Dahomey tombe après des guerres héroïques (Béhanzin, le roi requin, résista avec ses Agojié).",
                "1er août 1960 — indépendance. 1972 — Mathieu Kérékou : le Bénin devient le premier pays marxiste-léniniste d'Afrique, puis invente la Conférence nationale (1990) — le modèle de transition pacifique copié par toute l'Afrique.",
                "Le vodun béninois (Fon, Yoruba) est l'âme de la culture : de là partirent le candomblé du Brésil, le vaudou haïtien — l'Afrique dans les Amériques.",
            ],
            present: "Le pays de la « bonne gouvernance » : alternances démocratiques exemplaires, Cotonou la ville des artisans, la Fondation Zinsou.",
            futur: "Le Bénin, berceau du vodun et de la mémoire de la traite, deviendra la destination du tourisme mémoriel africain — les Amériques reviendront à Ouidah.",
        },
        FichePays {
            nom: "Togo", drapeau: "🇹🇬", capitale: "Lomé",
            ethnies: vec!["Éwé", "Kabyè", "Mina", "Tem", "Ntcham", "Gurma"],
            langues: vec!["Éwé", "Kabyè", "Mina", "Tem", "Ntcham"],
            histoire: vec![
                "Les Éwé du Notsé : la légende du roi Agokoli et la fuite du mur sacré — la migration fondatrice des Éwé vers la côte.",
                "1884 — le Togoland devient la colonie ALLEMANDE (la seule d'Afrique de l'Ouest) : plantations, chemin de fer de Lomé.",
                "1919 — partage franco-britannique après la défaite allemande. Le Togo français, le Togoland britannique (rejoint le Ghana en 1957).",
                "27 avril 1960 — indépendance avec Sylvanus Olympio, assassiné en 1963 — le premier coup d'État de l'Afrique indépendante.",
                "1967-2005 — Gnassingbé Eyadéma, 38 ans de pouvoir, l'armée des Kabyè. 2005 — Faure Gnassingbé succède à son père.",
            ],
            present: "Lomé, la ville frontière : le grand marché, le port franc, le siège de la Banque Ouest-Africaine. Le Togo est le pont entre le Ghana et le Nigeria.",
            futur: "Le corridor Lomé-Ouagadougou (la route du Nord) fera du Togo l'artère de l'AES vers la mer.",
        },
        FichePays {
            nom: "Cameroun", drapeau: "🇨🇲", capitale: "Yaoundé",
            ethnies: vec!["Bamiléké", "Bamoun", "Beti-Pahuin (Fang)", "Douala", "Peul", "Haoussa", "Kirdi", "Maka"],
            langues: vec!["Français", "Anglais", "Ewondo (Beti)", "Douala", "Bamoun", "Bamiléké (fe'fe')", "Fulfulde", "Haoussa"],
            histoire: vec![
                "Le royaume Bamoun : Njoya, le roi lettré, inventa son propre alphabet (shu-mom) au XIXe siècle — l'Afrique écrivait sa langue par elle-même.",
                "1884 — le Kamerun allemand. 1916 — partage français-britannique : le Cameroun devient le pays des deux colonisateurs, d'où sa double langue.",
                "1er janvier 1960 — indépendance française. 1961 — le Cameroun britannique du Sud rejoint la République : le Cameroun devient bilingue, « l'Afrique en miniature ».",
                "Ahidjo, puis Paul Biya depuis 1982 — un des plus longs règnes du monde.",
                "Les Lions Indomptables : la coupe d'Afrique 1984, 1988, 2000, 2002, 2017 — et le quart de finale du Mondial 1990, Roger Milla à 38 ans.",
            ],
            present: "Le Cameroun nourrit : cacao, café, bois, pétrole. Douala le port, Yaoundé les sept collines, le makossa et le bikutsi.",
            futur: "L'Afrique en miniature deviendra le laboratoire de l'unité : francophone + anglophone + 250 langues locales sur AfriChain = le pont de toute l'Afrique.",
        },
        FichePays {
            nom: "Congo", drapeau: "🇨🇬", capitale: "Brazzaville",
            ethnies: vec!["Kongo", "Téké", "Mbochi", "Lari", "Sangha", "Yombe"],
            langues: vec!["Lingala", "Kikongo", "Kituba (munukutuba)", "Téké", "Lari", "Mbochi"],
            histoire: vec![
                "Le royaume Kongo (1390) : Manikongo, la cour de M'banza-Kongo, la monnaie nzimbu — un royaume en relation diplomatique avec le Portugal AVANT que la traite ne détruise tout. Affonso Ier, roi lettré, écrivit au roi du Portugal pour dénoncer l'esclavage dès 1526.",
                "1880 — Pierre Savorgnan de Brazza signe le traité avec les Téké (le seul traité négocié sans canon). La colonie devient Congo français, Brazzaville la capitale de l'Afrique Équatoriale Française.",
                "1944 — la Conférence de Brazzaville : de Gaulle y annonce la fin du code de l'indigénat — le début de la fin de la colonisation.",
                "15 août 1960 — indépendance avec Fulbert Youlou. Puis Ngouabi, le socialisme congolais, Sassou-Nguesso.",
            ],
            present: "Brazzaville et Kinshasa se regardent sur le fleuve Congo — les deux capitales les plus proches du monde. Le pétrole du Congo, la rumba congolaise (Papa Wemba, Koffi Olomidé).",
            futur: "Les deux Congos reliés par le pont-rail sur le Pool : un axe de 17 millions d'habitants, la plus grande agglomération fluviale d'Afrique.",
        },
        FichePays {
            nom: "RD Congo", drapeau: "🇨🇩", capitale: "Kinshasa",
            ethnies: vec!["Kongo", "Luba", "Lunda", "Mongo", "Rwanda (Banyarwanda)", "Ngala", "Zande", "Chokwe"],
            langues: vec!["Lingala", "Swahili", "Kikongo", "Tshiluba", "Mongo", "Français"],
            histoire: vec![
                "Les royaumes Luba et Lunda (XVIe siècle) : la mémoire par les mbudye (les hommes de mémoire), le roi par le principe du bilele — des états organisés avant Léopold.",
                "1885 — Léopold II, roi des Belges, s'approprie personnellement l'État Indépendant du Congo : 10 millions de morts, les mains coupées, le caoutchouc rouge. Le premier génocide colonial documenté — la CRISE DU COBALT d'aujourd'hui est la suite du même pillage.",
                "1908 — l'État devient Congo belge. 30 juin 1960 — indépendance. Patrice Lumumba, le premier Premier ministre : « Nous ne sommes plus vos singes. » Assassiné en 1961 avec la complicité belge et de la CIA — le martyr de l'indépendance africaine.",
                "1965-1997 — Mobutu Sese Seko, 32 ans de dictature, le Zaïre pillé.",
                "1996-2003 — les guerres des Grands Lacs : 5 millions de morts, l'Afrique du Sud et les armées de 9 pays — la plus grande guerre africaine, déguisée en conflit ethnique, en réalité une guerre pour le coltan, l'or, le cobalt.",
                "Le coltan du Congo est dans CHAQUE téléphone du monde. Le cobalt du Congo est dans CHAQUE batterie électrique. Sans le Congo, la tech mondiale s'arrête.",
            ],
            present: "Kinshasa, la plus grande ville francophone du monde (17 millions). La rumba, le ndombolo, Fally Ipupa. Un pays grand comme l'Europe de l'Ouest, coupé en deux par l'Est en guerre.",
            futur: "Le Congo est la clef : si le Congo transforme son coltan et son cobalt sur place, l'Afrique tient la tech mondiale. L'Inga, le plus grand potentiel hydroélectrique de la planète, peut éclairer tout le continent.",
        },
        FichePays {
            nom: "Tchad", drapeau: "🇹🇩", capitale: "N'Djamena",
            ethnies: vec!["Sara", "Arabe (Choa)", "Kanembou", "Ouaddaïen", "Toubou", "Hadjarai", "Massa", "Peul"],
            langues: vec!["Français", "Arabe tchadien", "Sara", "Kanouri", "Ouaddaïen", "Toubou", "Fulfulde"],
            histoire: vec![
                "Le Kanem-Bornou : un des plus anciens empires d'Afrique (VIIIe siècle), autour du lac Tchad — les Sao, puis les Kanembou, les mai (rois) de la dynastie Sefuwa. L'érudition de Bornou rivalisait avec Tombouctou.",
                "Le royaume du Ouaddaï (Wadai) résista aux Français jusqu'en 1909 — la colonisation ne l'a jamais vraiment digéré.",
                "1920 — conquête française : « le Tchad est une colonie de tirailleurs » — les tirailleurs sénégalais du Tchad libérèrent la France (le colonel Leclerc partit de Fort-Lamy pour le sahara en 1940-42).",
                "11 août 1960 — indépendance avec François Tombalbaye. Puis Hissène Habré (1982-90, condamné pour crimes contre l'humanité), puis Idriss Déby, 30 ans.",
                "2021 — Mahamat Idriss Déby succède à son père mort au front. Le Tchad reste la plaque tournante du Sahel et du lac Tchad.",
            ],
            present: "Le Tchad, carrefour du Sahel : le pétrole de Doba (la pipe Tchad-Cameroun), les chevaux du Ouaddaï, le lac Tchad qui rétrécit — la soif et l'héroïsme.",
            futur: "Refaire le Grand Tchad (l'eau du fleuve), reboiser le Sahel, et faire du Tchad le pont entre l'AES et l'Afrique centrale.",
        },
    ]
}

// Les 40 autres pays — données compactes (capitale, langues, une ligne d'histoire)
pub fn fiches_compactes() -> Vec<(&'static str, &'static str, &'static str, Vec<&'static str>, &'static str)> {
    // (nom, capitale, drapeau, langues, une ligne d'histoire)
    vec![
        ("Algérie", "Alger", "🇩🇿", vec!["Arabe", "Berbère (Tamazight)", "Français"], "1954-1962 : la guerre d'indépendance la plus dure — l'Algérie paya un million et demi de morts pour sa liberté. Le pétrole et le gaz du Sahara, la Casbah."),
        ("Angola", "Luanda", "🇦🇴", vec!["Portugais", "Kikongo", "Kimbundu", "Umbundu"], "La reine Nzinga résista aux Portugais au XVIIe siècle. 1975 : indépendance après 500 ans de présence portugaise. Le pétrole, la kizomba."),
        ("Botswana", "Gaborone", "🇧🇼", vec!["Anglais", "Setswana", "Kalanga"], "1966 : à l'indépendance, 12 km de routes goudronnées. Aujourd'hui un des pays les plus stables d'Afrique — les diamants bien gérés."),
        ("Burundi", "Gitega", "🇧🇮", vec!["Kirundi", "Français", "Swahili"], "Le royaume Burundi, un des plus anciens états d'Afrique centrale, résista jusqu'en 1903. Les tambours sacrés du Burundi — l'âme bat."),
        ("Cabo Verde", "Praia", "🇨🇻", vec!["Portugais", "Kriolu"], "Îles volcaniques de la morna (Cesária Évora) — creuset du métissage atlantique, indépendance en 1975 avec Amílcar Cabral, le théoricien de la libération."),
        ("Centrafrique", "Bangui", "🇨🇫", vec!["Sango", "Français", "Gbaya", "Banda"], "Le pays de l'Ubangi : Barthélemy Boganda, le père fondateur, rêvait des États-Unis d'Afrique latine. Indépendance en 1960."),
        ("Comores", "Moroni", "🇰🇲", vec!["Comorien (Shikomori)", "Arabe", "Français"], "Les îles aux parfums (ylang-ylang) — la reine Fatima bint Abubekr, les sultanes comoriennes : une des rares îles gouvernées par des femmes au XVIe siècle."),
        ("Djibouti", "Djibouti", "🇩🇯", vec!["Français", "Arabe", "Somali", "Afar"], "1977 : indépendance de la dernière colonie française d'Afrique de l'Est. La porte du détroit Bab-el-Mandeb — 30% du commerce mondial passe devant ses côtes."),
        ("Égypte", "Le Caire", "🇪🇬", vec!["Arabe égyptien", "Arabe"], "Les pyramides, les hiéroglyphes, l'écriture inventée ici — 5000 ans d'état continu. L'Égypte est la mère de la mémoire écrite de l'humanité."),
        ("Érythrée", "Asmara", "🇪🇷", vec!["Tigrigna", "Arabe", "Tigré"], "1993 : indépendance après 30 ans de guerre contre l'Éthiopie — le pays le plus jeune d'Afrique par l'indépendance. Asmara, la ville Art déco."),
        ("Eswatini", "Mbabane", "🇸🇿", vec!["Anglais", "Swati"], "Le dernier royaume absolu d'Afrique — l'Umhlanga (la danse des roseaux), la culture Nguni préservée."),
        ("Éthiopie", "Addis-Abeba", "🇪🇹", vec!["Amharique", "Oromo", "Tigrigna", "Somali", "Afar"], "Le seul pays d'Afrique jamais colonisé — Adwa 1896 : Ménélik II écrase l'Italie. Lalibela, Axoum, les obélisques — 3000 ans d'état. Le café est né à Kaffa."),
        ("Gabon", "Libreville", "🇬🇦", vec!["Français", "Fang", "Myènè"], "La forêt océane : 88% du pays est forestier. L'okoumé, les masques Fang (qui inspirèrent Picasso), Léon Mba, indépendance 1960."),
        ("Gambie", "Banjul", "🇬🇲", vec!["Anglais", "Mandinka", "Wolof", "Fula"], "La plus petite country d'Afrique continentale — un fleuve, un pays. Kunta Kinteh (l'île James) : la mémoire de Roots, la traite atlantique."),
        ("Kenya", "Nairobi", "🇰🇪", vec!["Swahili", "Anglais", "Kikuyu", "Luo", "Kamba"], "La Mau Mau vainquit l'empire britannique (1952-60). Le M-Pesa : le Kenya inventa la monnaie mobile mondiale. Le berceau de l'humanité (Rift Valley)."),
        ("Lesotho", "Maseru", "🇱🇸", vec!["Sesotho", "Anglais"], "Le royaume de la montagne : le Basutoland de Moshoeshoe Ier, qui bâtit une nation dans les Maloti pour résister aux Zoulous et aux Boers."),
        ("Liberia", "Monrovia", "🇱🇷", vec!["Anglais", "Kpelle", "Bassa"], "1847 : la première république d'Afrique — fondée par les affranchis d'Amérique. Le drapeau étoilé, la constitution de 1847."),
        ("Libye", "Tripoli", "🇱🇾", vec!["Arabe", "Berbère", "Tamasheq"], "La Libye antique (les Garamantes du Fezzan), la route transsaharienne des caravanes. 1951 : indépendance (Idris Ier). 1969 : Kadhafi et la 3e théorie universelle."),
        ("Madagascar", "Antananarivo", "🇲🇬", vec!["Malgache", "Français"], "L'île-continent : les Vazimba, le riz en terrasses, le baobab, le famadihana (le retournement des morts). Le malgache relie l'Afrique et l'Asie du Sud-Est."),
        ("Malawi", "Lilongwe", "🇲🇼", vec!["Chichewa", "Anglais", "Yao", "Tumbuka"], "Le lac Nyasa (le lac des étoiles). 1964 : indépendance avec Hastings Kamuzu Banda. David Livingstone y trouva la lumière."),
        ("Maroc", "Rabat", "🇲🇦", vec!["Arabe", "Tamazight", "Français"], "Fès, la plus ancienne université du monde (Al-Qarawiyyin, 859, fondée par Fatima al-Fihri). Les dynasties almoravides et almohades gouvernèrent l'Espagne et le Sahel."),
        ("Maurice", "Port-Louis", "🇲🇺", vec!["Créole mauricien", "Français", "Anglais", "Bhojpuri"], "1968 : indépendance. Le dodo, le multiculturalisme hindou-creole-musulman-chinois — l'île de l'harmonie."),
        ("Mauritanie", "Nouakchott", "🇲🇷", vec!["Arabe", "Wolof", "Peul (Fulfulde)", "Soninké"], "Le berceau de l'Empire du Ghana (Wagadu) : les Soninké bâtirent Koumbi Saleh, la capitale de l'or. Chinguetti, la bibliothèque du désert."),
        ("Mozambique", "Maputo", "🇲🇿", vec!["Portugais", "Makhuwa", "Sena", "Swahili"], "1975 : indépendance avec Samora Machel (« la mort de l'exploiteur »). L'île de Mozambique, la route des Swahili, le marimba."),
        ("Namibie", "Windhoek", "🇳🇦", vec!["Anglais", "Oshiwambo", "Afrikaans", "Herero"], "1990 : la dernière colonie d'Afrique libérée. Le génocide des Herero et Nama (1904-08) — le premier génocide du XXe siècle, commis par l'Allemagne."),
        ("Rwanda", "Kigali", "🇷🇼", vec!["Kinyarwanda", "Français", "Anglais", "Swahili"], "Le royaume du mwami, la danse intore. 1994 : le génocide des Tutsi — 800 000 morts en 100 jours. Puis la réconciliation la plus rapide de l'histoire : « Ndi Umunyarwanda. »"),
        ("Sao Tomé-et-Principe", "São Tomé", "🇸🇹", vec!["Portugais", "Forro", "Principense"], "1470 : les premières îles « découvertes » de l'Atlantique sud — le premier régime de plantation sucrière du monde (le prototype de l'esclavage colonial)."),
        ("Sierra Leone", "Freetown", "🇸🇱", vec!["Anglais", "Krio", "Mende", "Temne"], "1787 : Freetown, la ville des affranchis. Bai Bureh résista à la taxe de case (1898). Le diamant, le Sierra Leone's Refugee All Stars."),
        ("Somalie", "Mogadiscio", "🇸🇴", vec!["Somali", "Arabe"], "Les cités-états swahilis du Nord (Zeila, Berbera), Sayyid Mohammed Abdullah Hassan (le Mad Mullah) résista 20 ans aux Britanniques. La poésie somalie, l'art verbal le plus riche d'Afrique."),
        ("Soudan", "Khartoum", "🇸🇩", vec!["Arabe", "Anglais", "Nubien", "Fur", "Beja"], "La Nubie : les pharaons noirs (la 25e dynastie), Méroé et ses pyramides (plus nombreuses qu'en Égypte). 1956 : indépendance. Le Nil, le coton, le Soudan des révolutions (2019)."),
        ("Soudan du Sud", "Djouba", "🇸🇸", vec!["Anglais", "Arabe juba", "Dinka", "Nuer"], "2011 : le pays le plus jeune du monde — l'indépendance après 50 ans de guerre. Le Nil Blanc, le bétail des Dinka."),
        ("Tanzanie", "Dodoma", "🇹🇿", vec!["Swahili", "Anglais"], "Le swahili, la langue de l'unité (Baraza la Kiswahili). Julius Nyerere, le Mwalimu (le professeur) : l'ujaama, la fierté socialiste. Kilimandjaro, Zanzibar, Serengeti."),
        ("Tunisie", "Tunis", "🇹🇳", vec!["Arabe", "Français"], "Carthage : Didon fonda la ville, Hannibal traversa les Alpes avec les éléphants. 1956 : Habib Bourguiba, la première indépendance du Maghreb. La jasmin révolution (2011)."),
        ("Ouganda", "Kampala", "🇺🇬", vec!["Anglais", "Luganda", "Swahili", "Runyankole"], "Le royaume Buganda (les kabaka), les sources du Nil (Speke, 1862). 1962 : indépendance. Idi Amin, puis la reconstruction. Les gorilles de Bwindi."),
        ("Zambie", "Lusaka", "🇿🇲", vec!["Anglais", "Bemba", "Nyanja", "Tonga"], "1964 : Kenneth Kaunda, « KK », l'indépendance de la Rhodésie du Nord. Le cuivre du Copperbelt, les chutes Victoria (Mosi-oa-Tunya, la fumée qui gronde)."),
        ("Zimbabwe", "Harare", "🇿🇼", vec!["Anglais", "Shona", "Ndebele"], "Le Grand Zimbabwe : la cité de pierre (XIe-XVe siècle), le Monomotapa, l'or des Shona. 1980 : Robert Mugabe. Les Chinhoyi caves, le mbira, les sculptures Shona."),
    ]
}

// ===== LES AI AUTONOMES DES LANGUES =====

#[derive(Clone)]
pub struct MotLangue {
    pub mot: String,
    pub sens: String,
    pub langue: String,
    pub pays: String,
    pub village: String,
    pub collecteur: String,
    pub ancien: String, // l'ancien qui a transmis le mot
    pub ts: u64,
}

impl MotLangue {
    pub fn to_json(&self) -> JsonValue {
        let mut m = HashMap::new();
        m.insert("mot".to_string(), JsonValue::Str(self.mot.clone()));
        m.insert("sens".to_string(), JsonValue::Str(self.sens.clone()));
        m.insert("langue".to_string(), JsonValue::Str(self.langue.clone()));
        m.insert("pays".to_string(), JsonValue::Str(self.pays.clone()));
        m.insert("village".to_string(), JsonValue::Str(self.village.clone()));
        m.insert("collecteur".to_string(), JsonValue::Str(self.collecteur.clone()));
        m.insert("ancien".to_string(), JsonValue::Str(self.ancien.clone()));
        m.insert("ts".to_string(), JsonValue::Int(self.ts as i64));
        JsonValue::Object(m)
    }
}

#[derive(Clone)]
pub struct AiLangue {
    pub langue: String,
    pub pays: String,
    pub mots: Vec<MotLangue>,
}

impl AiLangue {
    pub fn nouvelle(langue: &str, pays: &str) -> Self {
        AiLangue { langue: langue.to_string(), pays: pays.to_string(), mots: Vec::new() }
    }

    /// Le niveau de l'AI grandit avec les mots qu'elle mange.
    pub fn niveau(&self) -> (&'static str, &'static str, u64) {
        let n = self.mots.len() as u64;
        match n {
            0 => ("Nouveau-né", "🐣", 0),
            1..=9 => ("Enfant", "🌱", n),
            10..=49 => ("Adolescente", "🌿", n),
            50..=199 => ("Adulte", "🌳", n),
            _ => ("Sage", "👑", n),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let mut m = HashMap::new();
        m.insert("langue".to_string(), JsonValue::Str(self.langue.clone()));
        m.insert("pays".to_string(), JsonValue::Str(self.pays.clone()));
        m.insert("mots".to_string(), JsonValue::Array(self.mots.iter().map(|x| x.to_json()).collect()));
        JsonValue::Object(m)
    }
}

// ===== LE MAGASIN DE MÉMOIRE — persistance memoire.json =====

pub struct MemoireStore {
    pub langues: Vec<AiLangue>,
}

impl MemoireStore {
    pub fn chemin_data() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/afririch/memoire.json", home)
    }

    pub fn nouveau() -> Self {
        // On sème les AI des langues des 14 pays fondateurs — elles naissent
        // toutes ensemble, classées par ordre alphabétique des pays.
        let mut langues: Vec<AiLangue> = Vec::new();
        for f in fiches_pays() {
            for l in f.langues {
                langues.push(AiLangue::nouvelle(l, f.nom));
            }
        }
        langues.sort_by(|a, b| a.pays.cmp(&b.pays).then(a.langue.cmp(&b.langue)));
        MemoireStore { langues }
    }

    pub fn charger() -> Self {
        let chemin = Self::chemin_data();
        if let Ok(data) = std::fs::read_to_string(&chemin) {
            if let Ok(v) = from_str(&data) {
                return Self::from_json(&v);
            }
        }
        Self::nouveau()
    }

    pub fn sauvegarder(&self) {
        let _ = std::fs::write(Self::chemin_data(), to_string(&self.to_json()));
    }

    pub fn to_json(&self) -> JsonValue {
        let mut m = HashMap::new();
        m.insert("langues".to_string(), JsonValue::Array(self.langues.iter().map(|l| l.to_json()).collect()));
        JsonValue::Object(m)
    }

    pub fn from_json(v: &JsonValue) -> Self {
        let mut store = Self::nouveau();
        if let Some(obj) = v.as_object() {
            if let Some(JsonValue::Array(arr)) = obj.get("langues") {
                for l in arr {
                    if let Some(lo) = l.as_object() {
                        let langue = lo.get("langue").and_then(|x| x.as_str()).unwrap_or("").to_string();
                        let pays = lo.get("pays").and_then(|x| x.as_str()).unwrap_or("").to_string();
                        if langue.is_empty() { continue; }
                        let mut ai = AiLangue::nouvelle(&langue, &pays);
                        if let Some(JsonValue::Array(mots)) = lo.get("mots") {
                            for mo in mots {
                                if let Some(mo) = mo.as_object() {
                                    ai.mots.push(MotLangue {
                                        mot: mo.get("mot").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                        sens: mo.get("sens").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                        langue: langue.clone(),
                                        pays: pays.clone(),
                                        village: mo.get("village").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                        collecteur: mo.get("collecteur").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                        ancien: mo.get("ancien").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                        ts: mo.get("ts").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                    });
                                }
                            }
                        }
                        // Remplacer l'AI semée par la version nourrie
                        if let Some(pos) = store.langues.iter().position(|x| x.langue == langue && x.pays == pays) {
                            store.langues[pos] = ai;
                        } else {
                            store.langues.push(ai);
                        }
                    }
                }
            }
        }
        store.langues.sort_by(|a, b| a.pays.cmp(&b.pays).then(a.langue.cmp(&b.langue)));
        store
    }

    /// Nourrir une AI : le jeune collecte un mot d'un ancien → l'AI le mange.
    pub fn nourrir(&mut self, langue: &str, pays: &str, mot: &str, sens: &str, village: &str, collecteur: &str, ancien: &str) -> MotLangue {
        let m = MotLangue {
            mot: mot.to_string(), sens: sens.to_string(), langue: langue.to_string(),
            pays: pays.to_string(), village: village.to_string(), collecteur: collecteur.to_string(),
            ancien: ancien.to_string(), ts: now_timestamp() as u64,
        };
        if let Some(ai) = self.langues.iter_mut().find(|x| x.langue.eq_ignore_ascii_case(langue) && x.pays.eq_ignore_ascii_case(pays)) {
            ai.mots.push(m.clone());
        } else {
            // Nouvelle langue découverte par le peuple → nouvelle AI naît
            let mut ai = AiLangue::nouvelle(langue, pays);
            ai.mots.push(m.clone());
            self.langues.push(ai);
            self.langues.sort_by(|a, b| a.pays.cmp(&b.pays).then(a.langue.cmp(&b.langue)));
        }
        m
    }

    pub fn total_mots(&self) -> usize {
        self.langues.iter().map(|l| l.mots.len()).sum()
    }
}

// ===== LA RECHERCHE — SAHARA AFRI, le Google qui connaît l'Afrique =====

/// Chercher un pays par nom (insensible aux accents et à la casse).
pub fn chercher_pays(q: &str) -> Option<FichePays> {
    let nq = normaliser(q);
    if nq.is_empty() { return None; }
    for f in fiches_pays() {
        if normaliser(f.nom).contains(&nq) || nq.contains(&normaliser(f.nom)) {
            return Some(f);
        }
    }
    None
}

/// Chercher dans les fiches compactes (les autres pays).
/// Retourne des valeurs possédées (pas de référence à des données locales).
pub fn chercher_pays_compact(q: &str) -> Option<(String, String, String, Vec<String>, String)> {
    let nq = normaliser(q);
    if nq.is_empty() { return None; }
    for (nom, cap, flag, langues, histoire) in fiches_compactes() {
        if normaliser(nom).contains(&nq) || nq.contains(&normaliser(nom)) {
            return Some((nom.to_string(), cap.to_string(), flag.to_string(), langues.iter().map(|l| l.to_string()).collect(), histoire.to_string()));
        }
    }
    None
}

fn normaliser(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        let rep = match c.to_ascii_lowercase() {
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'à' | 'â' => 'a',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'û' | 'ù' | 'ü' => 'u',
            'ç' => 'c',
            '\'' | '’' => ' ',
            other => other,
        };
        out.push(rep);
    }
    out.trim().to_lowercase()
}
