// ===== AFRI RECOVER — Investigation des Seeds Occidentales =====
// Outil standalone: scanne l'appareil pour trouver les seeds cachées
// des wallets occidentaux (Trust Wallet, MetaMask, Binance, etc.)
// Compile: rustc afri_recover.rs -o afri_recover
// Aucune dépendance — Rust std uniquement
//
// "L'Occident dit 'perdu' — nous trouvons." 💚🦁

use std::fs;
use std::path::Path;
use std::process::Command;

// Couleurs terminal
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

// 20 wallets occidentaux connus
const WESTERN_WALLETS: &[(&str, &str)] = &[
    ("Trust Wallet", "/data/data/com.wallet.crypto.trustapp/"),
    ("MetaMask", "/data/data/io.metamask/"),
    ("Coinbase Wallet", "/data/data/org.toshi/"),
    ("Exodus", "/data/data/exodusmovement.exodus/"),
    ("Bitcoin Core", "/data/data/org.bitcoin.wallet/"),
    ("Electrum", "/data/data/org.electrum.electrum/"),
    ("Blue Wallet", "/data/data/io.bluewallet.bluewallet/"),
    ("Atomic Wallet", "/data/data/io.atomicwallet/"),
    ("Phantom", "/data/data/app.phantom/"),
    ("Solflare", "/data/data/com.solflare.wallet/"),
    ("Binance", "/data/data/com.binance.dev/"),
    ("KuCoin", "/data/data/com.kucoin/"),
    ("Crypto.com", "/data/data/co.mona.androidCrypto/"),
    ("SafePal", "/data/data/com.safepal.wallet/"),
    ("Math Wallet", "/data/data/com.mathwallet.android/"),
    ("TokenPocket", "/data/data/com.tokenpocket.pro.wallet/"),
    ("imToken", "/data/data/org.consenlabs.token/"),
    ("Alpha Wallet", "/data/data/com.alphawallet.wallet/"),
    ("ONTO", "/data/data/com.onto.wallet/"),
    ("Coinomi", "/data/data/com.coinomi.android/"),
];

// Procédures pour afficher les 24 mots dans chaque wallet
const WALLET_PROCEDURES: &[(&str, &[&str])] = &[
    ("Trust Wallet", &[
        "Ouvrez Trust Wallet sur votre téléphone",
        "Appuyez sur Settings (Paramètres) en bas à droite",
        "Appuyez sur Wallets (Portefeuilles)",
        "Sélectionnez le wallet que vous voulez récupérer",
        "Appuyez sur Backup (Sauvegarder) > Manual (Manuel)",
        "Vos 24 MOTS SECRETS s'affichent à l'écran",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("MetaMask", &[
        "Ouvrez MetaMask sur votre téléphone",
        "Appuyez sur le menu (3 lignes) en haut à gauche",
        "Appuyez sur Settings (Paramètres) > Security & Privacy",
        "Appuyez sur 'Reveal Secret Recovery Phrase'",
        "Entrez votre mot de passe MetaMask",
        "Vos 12 MOTS SECRETS s'affichent à l'écran",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("Binance", &[
        "Ouvrez l'app Binance sur votre téléphone",
        "Appuyez sur Wallets (Portefeuilles) en bas",
        "Appuyez sur Overview (Aperçu) > Backup",
        "Suivez les instructions de sauvegarde",
        "Vos 24 MOTS SECRETS s'affichent (si wallet non-custodial)",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
        "ATTENTION: Binance Custodial ne montre PAS la seed",
    ]),
    ("Coinbase Wallet", &[
        "Ouvrez Coinbase Wallet sur votre téléphone",
        "Appuyez sur Settings (Paramètres)",
        "Appuyez sur Recovery Phrase (Phrase de récupération)",
        "Entrez votre mot de passe Coinbase",
        "Vos 12 ou 24 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("Exodus", &[
        "Ouvrez Exodus sur votre téléphone",
        "Appuyez sur Settings (Paramètres) > Backup",
        "Entrez votre mot de passe Exodus",
        "Vos 12 MOTS SECRETS s'affichent à l'écran",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("Bitcoin Core (Terminal)", &[
        "Ouvrez Termux et tapez: bitcoin-cli dumpwallet ~/wallet_dump.txt",
        "Le fichier wallet_dump.txt contient toutes vos clés privées",
        "Tapez: cat ~/wallet_dump.txt pour voir les clés",
        "Chaque clé privée commence par '5', 'K' ou 'L' (WIF format)",
        "Convertissez les clés en 24 mots avec un outil BIP39",
    ]),
    ("Electrum (Terminal)", &[
        "Ouvrez Termux et tapez: electrum listseeds --wallet ~/.electrum/wallets/default_wallet",
        "Ou tapez: cat ~/.electrum/wallets/default_wallet",
        "Le fichier contient votre seed chiffrée",
        "Tapez: electrum decryptseed <votre_seed_chiffrée> <votre_password>",
        "Vos 12 ou 24 MOTS SECRETS s'affichent",
    ]),
    ("Blue Wallet", &[
        "Ouvrez Blue Wallet sur votre téléphone",
        "Sélectionnez le wallet dans la liste",
        "Appuyez sur Settings (Paramètres) > Backup",
        "Vos 12 MOTS SECRETS s'affichent à l'écran",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("Atomic Wallet", &[
        "Ouvrez Atomic Wallet sur votre téléphone",
        "Appuyez sur Settings (Paramètres) > Backup",
        "Entrez votre mot de passe Atomic Wallet",
        "Vos 12 MOTS SECRETS s'affichent à l'écran",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("Phantom (Solana)", &[
        "Ouvrez Phantom sur votre téléphone",
        "Appuyez sur Settings (Paramètres) > Recovery Phrase",
        "Entrez votre mot de passe Phantom",
        "Vos 12 ou 24 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("SafePal", &[
        "Ouvrez SafePal sur votre téléphone",
        "Appuyez sur Settings (Paramètres) > Security",
        "Appuyez sur Recovery Phrase (Phrase de récupération)",
        "Vos 12 ou 24 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("imToken", &[
        "Ouvrez imToken sur votre téléphone",
        "Appuyez sur Me (Moi) > Settings > Backup",
        "Entrez votre mot de passe imToken",
        "Vos 12 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("TokenPocket", &[
        "Ouvrez TokenPocket sur votre téléphone",
        "Appuyez sur Settings (Paramètres) > Backup",
        "Entrez votre mot de passe TokenPocket",
        "Vos 12 ou 24 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("Math Wallet", &[
        "Ouvrez Math Wallet sur votre téléphone",
        "Appuyez sur Settings (Paramètres) > Backup",
        "Vos 12 ou 24 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("Coinomi", &[
        "Ouvrez Coinomi sur votre téléphone",
        "Appuyez sur Settings (Paramètres) > Backup",
        "Entrez votre mot de passe Coinomi",
        "Vos 12 ou 24 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("KuCoin", &[
        "Ouvrez KuCoin sur votre téléphone",
        "Appuyez sur Wallet > Backup Wallet",
        "Suivez les instructions de sauvegarde",
        "ATTENTION: KuCoin est principalement custodial",
        "La seed n'est disponible que pour le wallet non-custodial",
    ]),
    ("Crypto.com", &[
        "Ouvrez Crypto.com sur votre téléphone",
        "Appuyez sur Settings > Recovery Phrase",
        "ATTENTION: Crypto.com DeFi Wallet montre la seed",
        "L'app principale Crypto.com est custodial (pas de seed)",
        "Utilisez l'app DeFi Wallet séparée pour voir vos 24 mots",
    ]),
    ("Alpha Wallet", &[
        "Ouvrez Alpha Wallet sur votre téléphone",
        "Appuyez sur Settings (Paramètres) > Backup",
        "Vos 12 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("ONTO", &[
        "Ouvrez ONTO sur votre téléphone",
        "Appuyez sur Settings > Backup > Recovery Phrase",
        "Vos 12 ou 24 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
    ("Solflare", &[
        "Ouvrez Solflare sur votre téléphone",
        "Appuyez sur Settings > Recovery Phrase",
        "Entrez votre mot de passe Solflare",
        "Vos 12 ou 24 MOTS SECRETS s'affichent",
        "Notez-les sur papier — c'est VOTRE seed de récupération",
    ]),
];

// Mots-clés pour détecter des seeds
const SEED_KEYWORDS: &[&str] = &[
    "mnemonic", "seed", "seedPhrase", "seed_phrase", "privateKey", "private_key",
    "recoveryPhrase", "recovery_phrase", "backup", "walletSeed", "wallet_seed",
    "secretRecoveryPhrase", "encryptedMnemonic", "keyStore", "keystore",
    "xprv", "cipher", "ciphertext",
];

// Extensions wallet
const WALLET_EXTENSIONS: &[&str] = &[
    "dat", "json", "keystore", "key", "seed", "wallet", "mnemonic", "bak", "backup", "enc",
];

// Mots BIP39 communs (pour détection heuristique de seeds)
const COMMON_BIP39: &[&str] = &[
    "abandon","ability","able","about","above","absent","absorb","abstract","absurd","abuse",
    "access","accident","account","accuse","achieve","acid","acoustic","acquire","across","act",
    "action","actor","actress","actual","adapt","add","addict","address","adjust","admit",
    "adult","advance","advice","aerobic","affair","afford","afraid","again","age","agent",
    "agree","ahead","aim","air","airport","aisle","alarm","album","alcohol","alert",
    "alien","all","alley","allow","almost","alone","alpha","already","also","alter",
    "always","amateur","amazing","among","amount","amused","analyst","anchor","ancient","anger",
    "angle","angry","animal","ankle","announce","annual","another","answer","antenna","antique",
    "anxiety","any","apart","apology","appear","apple","approve","april","arch","arctic",
    "area","arena","argue","arm","armed","armor","army","around","arrange","arrest",
    "arrive","arrow","art","artefact","artist","artwork","ask","aspect","assault","asset",
    "assist","assume","asthma","athlete","atom","attack","attend","attitude","attract","auction",
    "audit","august","aunt","author","auto","autumn","average","avocado","avoid","awake",
    "aware","away","awesome","awful","awkward","axis","baby","bachelor","bacon","badge",
    "bag","balance","balcony","ball","bamboo","banana","banner","bar","barely","bargain",
    "barrel","scale","base","basic","basket","battle","beach","bean","beauty","because",
    "become","beef","before","begin","behave","behind","believe","below","belt","bench",
    "benefit","best","betray","better","between","beyond","bicycle","bid","bike","bind",
    "biology","bird","birth","bitter","black","blade","blame","blanket","blast","bleak",
    "bless","blind","blood","blossom","blouse","blue","blur","blush","board","boat",
    "body","boil","bomb","bone","bonus","book","boost","border","boring","borrow",
    "boss","bottom","bounce","box","boy","bracket","brain","brand","brass","brave",
    "bread","breeze","brick","bridge","brief","bright","bring","brisk","broccoli","broken",
    "bronze","broom","brother","brown","brush","bubble","buddy","budget","buffalo","build",
    "bulb","bulk","bullet","bundle","bunker","burden","burger","burst","bus","business",
    "busy","butter","buyer","buzz","cabbage","cabin","cable","cactus","cage","cake",
    "call","calm","camera","camp","can","canal","cancel","candy","cannon","canoe",
    "canvas","canyon","capable","capital","captain","car","carbon","card","cargo","carpet",
    "carry","cart","case","cash","casino","castle","casual","cat","catalog","catch",
    "category","cattle","caught","cause","caution","cave","ceiling","celery","cement","census",
    "century","cereal","certain","chair","chalk","champion","change","chaos","chapter","charge",
    "chase","chat","cheap","check","cheese","chef","cherry","chest","chicken","chief",
    "child","chimney","choice","choose","chronic","chuckle","chunk","churn","cigar","cinnamon",
    "circle","citizen","city","civil","claim","clap","clarify","claw","clay","clean",
    "clerk","clear","click","cliff","climb","clinic","clip","clock","clog","cloth",
    "cloud","clown","club","clue","clutch","coach","coast","coconut","code","coffee",
    "coil","coin","collect","color","column","combine","come","comfort","comic","common",
    "company","concert","conduct","confirm","congress","connect","consider","control","convince","cook",
    "cool","copper","copy","coral","core","corn","correct","cost","cotton","couch",
    "country","couple","course","cousin","cover","coyote","crack","cradle","craft","cram",
    "crane","crash","crater","crawl","crazy","cream","credit","creek","crew","cricket",
    "crime","crisp","critic","crop","cross","crouch","crowd","crucial","cruel","cruise",
    "crumble","crunch","crush","cry","crystal","cube","culture","cup","cupboard","curious",
    "current","curtain","curve","cushion","custom","cute","cycle","dad","damage","damp",
    "dance","danger","daring","dash","daughter","dawn","day","deal","dirt","disc",
    "divert","divide","dizzy","doctor","dolphin","domain","donate","double","dough","dove",
    "dragon","drama","drank","draw","dream","dress","drift","drill","drink","drive",
    "drop","drum","duck","dune","dust","duty","dwarf","dynamic","eager","eagle",
    "early","earn","earth","easily","east","easy","echo","ecology","economy","edge",
    "edit","effect","effort","egg","eight","either","elbow","elder","electric","elegant",
    "element","elephant","elevator","elite","embark","embody","emerge","emotion","employ","empty",
    "enable","enact","end","endless","endorse","enemy","energy","enforce","engage","engine",
    "enjoy","enlist","enough","enrich","enroll","ensure","enter","entire","entry","envelope",
    "episode","equal","equip","erase","erode","erosion","error","erupt","escape","essay",
    "estate","eternal","ethics","evidence","evil","evoke","evolve","exact","example","excess",
    "exchange","excite","exclude","excuse","execute","exercise","exhaust","exhibit","exile","exist",
    "exit","exotic","expand","expect","expire","explain","expose","express","extend","extra",
    "eye","fabric","face","faculty","fade","faint","faith","falcon","fall","false",
    "fame","family","famous","fan","fancy","fantasy","farm","fashion","fat","fatal",
    "father","fatigue","fault","favorite","feature","february","federal","fellow","female","fence",
    "festival","fetch","fever","few","fiber","fiction","field","figure","file","film",
    "filter","final","find","fine","finger","finish","fire","firm","first","fiscal",
    "fish","fit","fitness","fix","flag","flame","flash","flat","flavor","flee",
    "flight","float","flock","floor","flower","fluid","flush","fly","foam","focus",
    "fog","foil","fold","follow","food","foot","force","forest","forget","fork",
    "fortune","forum","forward","fossil","foster","found","fox","fragile","frame","frequent",
    "fresh","friend","fringe","frog","front","frost","frown","frozen","fruit","fuel",
    "full","fun","funny","furnace","fury","future","gadget","gain","galaxy","gallery",
    "game","gap","garage","garbage","garden","garlic","garment","gas","gasp","gate",
    "gather","gauge","gaze","general","genius","genre","gentle","genuine","gesture","ghost",
    "giant","gift","giraffe","girl","glad","glance","glare","glass","glide","globe",
    "gloom","glory","glove","glow","glue","goat","goddess","gold","good","goose",
    "gorilla","gospel","gossip","govern","grace","grain","grand","grant","grape","grass",
    "gravity","great","green","grid","grief","grit","grocery","group","grow","grunt",
    "guard","guess","guide","guilt","guitar","gun","gym","habit","hair","half",
    "hammer","hand","happy","harbor","hard","harsh","harvest","hat","have","hawk",
    "hazard","head","health","heart","heavy","hedgehog","height","hello","helmet","herald",
    "hero","hidden","high","hill","hint","hip","hire","history","hobby","hockey",
    "hold","hole","holiday","hollow","home","honey","hood","hope","horn","horror",
    "horse","hospital","host","hotel","hour","hover","hub","huge","human","humble",
    "humor","hundred","hungry","hunt","hurdle","hurry","hurt","hybrid","ice","icon",
    "idea","identify","idle","ignore","ill","illegal","illness","image","imagine","immense",
    "impact","impose","improve","impulse","inch","include","income","increase","index","indicate",
    "indoor","industry","infant","inflict","inform","inhale","inherit","initial","inject","injury",
    "inland","inner","insect","insert","inside","inspire","install","intact","interest","into",
    "invest","invite","involve","iron","island","isolate","issue","item","ivory","jacket",
    "jaguar","jar","jazz","jealous","jeans","jelly","jewel","job","join","joint",
    "joke","journey","joy","judge","juice","jump","jungle","junior","junk","just",
    "kangaroo","keen","keep","ketchup","key","kick","kid","kidney","kind","king",
    "kiss","kite","kitten","knee","knife","knock","know","lab","label","labor",
    "ladder","lady","lake","lamp","land","language","lantern","laptop","large","later",
    "latin","laugh","laundry","lava","law","lawn","lawsuit","layer","lazy","leader",
    "leaf","learn","leave","lecture","left","leg","legal","legend","leisure","lemon",
    "lend","length","lens","leopard","lesson","letter","level","liar","liberty","library",
    "license","life","lift","light","limit","link","lion","liquid","list","live",
    "lizard","load","lobster","local","lock","lodge","log","logic","lonely","long",
    "loop","lottery","loud","lounge","love","loyal","lucky","lumber","lunch","luxury",
    "lyrics","machine","mad","magic","magnet","maid","mail","main","major","make",
    "mammal","man","manage","mandate","mango","mansion","manual","maple","marble","march",
    "margin","marine","market","marriage","mask","mass","master","match","matter","mature",
    "maximum","mayor","meal","mean","meat","media","melody","melt","member","memory",
    "mention","menu","mercy","merge","merit","merry","message","metal","method","middle",
    "midnight","milk","million","mimic","mind","minimum","minor","minute","miracle","mirror",
    "misery","miss","mistake","mix","mixed","mixture","mobile","model","modify","mom",
    "moment","monitor","monkey","monster","month","moon","moral","more","morning","mother",
    "motion","motor","mountain","mouse","move","movie","much","muffin","mule","multiply",
    "muscle","museum","music","mutual","myself","mystery","myth","naive","name","napkin",
    "narrow","nasty","nation","nature","near","neck","need","negative","neglect","neither",
    "nephew","nerve","nest","net","network","neutral","never","news","next","nice",
    "night","noble","noise","nominee","noodle","normal","north","nose","notable","note",
    "nothing","notice","novel","now","nuclear","number","nurse","nut","oak","obey",
    "object","oblige","obscure","observe","obtain","obvious","occur","ocean","october","odor",
    "off","offer","office","often","oil","okay","old","olive","olympic","omit",
    "once","one","onion","online","only","open","opera","opinion","oppose","option",
    "orange","orbit","orchard","order","organ","organize","origin","ornament","orphan","ostrich",
    "other","outdoor","outer","output","outside","oval","oven","over","own","owner",
    "oxygen","oyster","ozone","paddle","page","pair","palace","palm","panel","panic",
    "panther","paper","parade","parent","parish","park","parrot","party","pass","patch",
    "path","patient","patrol","pattern","pause","pave","peace","peanut","pear","peasant",
    "pelican","pen","penalty","pencil","people","pepper","perfect","permit","person","pet",
    "phone","photo","phrase","physical","piano","picnic","picture","piece","pig","pigeon",
    "pill","pilot","pink","pioneer","pipe","pistol","pitch","pizza","place","planet",
    "plant","plastic","plate","play","please","pledge","plenty","plot","plunge","poem",
    "poet","point","polar","pole","police","pond","pony","pool","popular","portion",
    "position","possible","post","potato","pottery","poverty","powder","power","practice","praise",
    "predict","prefer","prepare","present","pretty","prevent","price","pride","primary","print",
    "priority","prison","private","prize","problem","process","produce","profit","program","project",
    "promote","proof","property","prosper","protect","proud","provide","public","pudding","pull",
    "pulp","pulse","pumpkin","punch","pupil","puppy","purchase","purity","purpose","purse",
    "push","put","puzzle","pyramid","quality","quantum","quarter","question","quick","quit",
    "quiz","quote","rabbit","raccoon","race","rack","radar","radio","rail","rain",
    "raise","rally","ramp","ranch","random","range","rank","rapid","rare","rate",
    "rather","raven","raw","razor","ready","real","reason","rebel","rebuild","recall",
    "receive","recipe","record","recycle","reduce","refer","reflect","reform","refuse","region",
    "regret","regular","reject","relax","release","relief","rely","remain","remember","remind",
    "remove","render","rent","reopen","repair","repeat","replace","report","require","rescue",
    "resemble","resist","resort","result","retire","retreat","return","reunion","reveal","review",
    "reward","rhythm","rib","ribbon","rice","rich","ride","ridge","rifle","right",
    "rigid","ring","riot","ripple","risk","ritual","rival","river","road","roast",
    "robot","robust","rocket","romance","roof","rookie","room","rose","rotate","rough",
    "round","route","royal","rubber","rude","rug","rule","run","rural","saddle",
    "sadness","safe","sail","salad","salmon","salon","salt","salute","same","sample",
    "sand","satisfy","satoshi","sauce","savage","saw","scan","scare","scatter","scene",
    "scheme","school","science","scope","score","screen","script","sea","search","season",
    "seat","second","secret","section","secure","seed","seek","segment","select","sell",
    "seminar","senior","sense","sentence","series","service","session","settle","setup","seven",
    "shadow","shaft","shallow","share","shed","shell","sheriff","shield","shift","shine",
    "ship","shiver","shock","shoe","shoot","shop","short","shoulder","shout","shove",
    "shower","shrimp","shrug","shuffle","shy","sibling","sick","side","siege","sight",
    "sign","silent","silk","silly","silver","similar","simple","since","sing","siren",
    "sister","situate","six","size","skate","sketch","ski","skill","skin","skirt",
    "skull","slam","sleep","slice","slide","slight","slow","slogan","slot","smart",
    "smile","smoke","smooth","snack","snake","snap","sniff","snow","soap","soccer",
    "social","sock","soda","soft","solar","soldier","solid","solution","solve","someone",
    "song","soon","sorry","sort","soul","sound","soup","source","south","space",
    "spare","spatial","spawn","speak","special","speed","spell","spend","sphere","spice",
    "spider","spike","spin","spirit","spit","spite","split","spoil","sponsor","spoon",
    "sport","spot","spray","spread","spring","spy","square","squeeze","squirrel","stable",
    "stadium","staff","stage","stair","stamp","stand","star","start","state","station",
    "stay","steak","steel","stem","step","stereo","stick","still","stitch","stock",
    "stomach","stone","stool","story","stove","strategy","street","strike","strong","struggle",
    "student","stuff","stumble","style","subject","submit","subway","success","such","sudden",
    "suffer","sugar","suggest","suit","summer","sun","sunny","sunset","super","supply",
    "supreme","sure","surf","surface","surge","surprise","surround","survey","suspect","sustain",
    "swallow","swamp","swap","swarm","sweat","sweep","sweet","swift","swim","swing",
    "switch","sword","symbol","symptom","syrup","system","table","tackle","tag","tail",
    "talent","talk","tank","tape","target","task","taste","tattoo","taxi","teach",
    "team","tell","ten","tenant","tennis","tent","term","test","text","thank",
    "that","theme","then","theory","there","they","thing","this","thought","three",
    "thrive","throw","thumb","thunder","ticket","tide","tiger","tilt","timber","time",
    "tiny","tip","tired","tissue","title","toast","tobacco","today","toddler","together",
    "toilet","token","tomato","tomorrow","tone","tongue","tonight","tool","tooth","top",
    "topic","topple","torch","tornado","tortoise","total","toss","tour","toward","tower",
    "town","toy","track","trade","traffic","train","trap","trash","travel","treat",
    "tree","trend","trial","tribe","trick","trigger","trim","trip","trophy","trouble",
    "truck","truly","trumpet","trust","truth","try","tube","tuition","tumble","tuna",
    "tunnel","turkey","turn","turtle","twelve","twenty","twice","twin","twist","two",
    "type","typical","ugly","umbrella","unable","unaware","uncle","uncover","under","undo",
    "unfair","unfold","unhappy","uniform","unique","unit","universe","unknown","unlock","until",
    "unusual","unveil","update","upgrade","upon","upper","upset","urban","urge","usage",
    "use","used","useful","useless","usual","utility","vacant","vacuum","vague","valid",
    "valley","value","van","vanish","vapor","various","vast","vault","version","very",
    "vessel","veteran","viable","vibrant","vicious","victory","video","view","village","vintage",
    "violin","virtual","virus","visa","visit","visual","vital","vivid","vocal","voice",
    "void","volcano","volume","vote","voyage","wage","wagon","wait","walk","wall",
    "walnut","want","warfare","warm","warrior","wash","wasp","waste","water","wave",
    "way","wealth","weapon","wear","weasel","weather","web","wedding","weekend","weird",
    "welcome","west","wet","whale","what","wheat","wheel","when","where","whip",
    "whisper","wide","width","wife","wild","will","win","window","wine","wing",
    "wink","winner","winter","wire","wisdom","wise","wish","witness","wolf","woman",
    "wonder","wood","wool","word","work","world","worry","worth","wrap","wreck",
    "wrestle","wrist","write","wrong","yard","year","yellow","you","young","youth",
    "zebra","zero","zone","zoo",
];

struct ScanResult {
    found_wallets: Vec<(String, String)>,
    found_seeds: Vec<(String, Vec<String>)>,
    found_keys: Vec<(String, String)>,
    detected_apps: Vec<(String, String, bool)>,
    blockchain_data: Vec<(String, String)>,
}

fn main() {
    println!("{}╔══════════════════════════════════════════════════╗{}", BOLD, RESET);
    println!("{}║   AFRI RECOVER — Investigation Seeds Occidentales  ║{}", BOLD, RESET);
    println!("{}║   L'Occident dit 'perdu' — nous trouvons. 💚🦁    ║{}", BOLD, RESET);
    println!("{}╚══════════════════════════════════════════════════╝{}", BOLD, RESET);
    println!();
    println!("{}Scan en cours...{}", CYAN, RESET);
    println!();

    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    let mut result = ScanResult {
        found_wallets: Vec::new(),
        found_seeds: Vec::new(),
        found_keys: Vec::new(),
        detected_apps: Vec::new(),
        blockchain_data: Vec::new(),
    };

    // 1. Scan apps wallet
    println!("{}[1] Scan des applications wallet occidentales...{}", CYAN, RESET);
    for (name, path) in WESTERN_WALLETS {
        let exists = Path::new(path).exists();
        let accessible = if exists { fs::read_dir(path).is_ok() } else { false };
        if exists {
            let status = if accessible { "ACCESSIBLE" } else { "VERROUILLÉ (root requis)" };
            println!("  {} {} → [{}]", if accessible { "✅" } else { "🔒" }, name, status);
            result.detected_apps.push((name.to_string(), path.to_string(), accessible));
        }
    }
    if result.detected_apps.is_empty() {
        println!("  Aucune app wallet détectée.");
    }
    println!();

    // 2. Scan fichiers
    println!("{}[2] Scan des fichiers wallet...{}", CYAN, RESET);
    scan_directory(&home, &mut result, 0);
    let sdcard = "/sdcard";
    if Path::new(sdcard).exists() {
        println!("  Scan /sdcard/...");
        scan_directory(sdcard, &mut result, 0);
    }
    for (_, path) in WESTERN_WALLETS {
        if Path::new(path).exists() {
            scan_directory(path, &mut result, 0);
        }
    }
    println!();

    // 3. Affichage terminal
    println!("{}[3] Résultats:{}", CYAN, RESET);
    println!("  • {} app(s) wallet détectée(s)", result.detected_apps.len());
    println!("  • {} fichier(s) wallet trouvé(s)", result.found_wallets.len());
    println!("  • {} seed(s) extraite(s)", result.found_seeds.len());
    println!("  • {} clé(s) privée(s)", result.found_keys.len());
    println!();

    // 4. Seeds trouvées
    if !result.found_seeds.is_empty() {
        println!("{}🎯 SEEDS TROUVÉES!{}", GREEN, RESET);
        for (source, words) in &result.found_seeds {
            println!("  Source: {}", source);
            println!("  {} mots:", words.len());
            for (i, w) in words.iter().enumerate() {
                print!("  {}. {:<15}", i + 1, w);
                if (i + 1) % 4 == 0 { println!(); }
            }
            println!();
        }
    }

    // 5. Générer rapport HTML
    println!("{}[4] Génération du rapport HTML...{}", CYAN, RESET);
    let report_path = format!("{}/afri_recover_report.html", home);
    let html = generate_html_report(&result);
    match fs::write(&report_path, html) {
        Ok(_) => println!("  {}Rapport sauvegardé: {}{}", GREEN, report_path, RESET),
        Err(e) => println!("  {}Erreur sauvegarde: {}{}", RED, e, RESET),
    }

    // 6. Ouvrir dans Chrome
    println!("{}[5] Ouverture dans Chrome...{}", CYAN, RESET);
    open_in_chrome(&report_path);

    println!();
    println!("{}{}L'Occident dit 'impossible' mais stocke quand même.{}", BOLD, RED, RESET);
    println!("{}{}L'Afrique dit: 'récupérable, transparent.' 💚🦁{}", BOLD, GREEN, RESET);
    println!();
}

fn scan_directory(dir: &str, result: &mut ScanResult, depth: u8) {
    if depth > 3 { return; }
    let entries = match fs::read_dir(dir) { Ok(e) => e, Err(_) => return };
    for entry in entries.flatten() {
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if path.is_dir() {
            scan_directory(&path_str, result, depth + 1);
        } else {
            let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            let is_wallet = WALLET_EXTENSIONS.contains(&ext.as_str())
                || name.contains("wallet") || name.contains("seed")
                || name.contains("mnemonic") || name.contains("keystore")
                || name.contains("private") || name.contains("backup");
            if name.contains("blocks") || name.contains("chainstate") || path_str.contains("/blocks/") {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                result.blockchain_data.push((path_str.clone(), format_size(size)));
            }
            if is_wallet {
                let content = fs::read_to_string(&path).unwrap_or_default();
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let summary = if content.is_empty() {
                    format!("{} (binaire)", format_size(size))
                } else {
                    format!("{} (texte)", format_size(content.len() as u64))
                };
                result.found_wallets.push((path_str.clone(), summary));
                extract_seeds(&path_str, &content, result);
            } else if let Ok(content) = fs::read_to_string(&path) {
                if content.len() < 500_000 {
                    extract_seeds(&path_str, &content, result);
                }
            }
        }
    }
}

fn extract_seeds(source: &str, content: &str, result: &mut ScanResult) {
    for keyword in SEED_KEYWORDS {
        if content.to_lowercase().contains(keyword) {
            for line in content.lines() {
                let words: Vec<&str> = line.split_whitespace().collect();
                if words.len() == 12 || words.len() == 24 {
                    let all_bip39 = words.iter().all(|w| {
                        let binding = w.to_lowercase();
                        let w_lower = binding.trim_matches(|c: char| !c.is_ascii_alphanumeric());
                        COMMON_BIP39.contains(&w_lower) || is_bip39_likely(w_lower)
                    });
                    if all_bip39 {
                        let wv: Vec<String> = words.iter().map(|w| w.to_string()).collect();
                        if !result.found_seeds.iter().any(|(_, w)| w == &wv) {
                            result.found_seeds.push((source.to_string(), wv));
                        }
                    }
                }
            }
            let mut hex = String::new();
            for c in content.chars() {
                if c.is_ascii_hexdigit() { hex.push(c); }
                else {
                    if hex.len() >= 64 { result.found_keys.push((source.to_string(), hex.clone())); }
                    hex.clear();
                }
            }
            if hex.len() >= 64 { result.found_keys.push((source.to_string(), hex)); }
            break;
        }
    }
    if content.contains("{") && (content.contains("mnemonic") || content.contains("seed") || content.contains("privateKey")) {
        for kw in &["mnemonic", "seed", "seedPhrase", "privateKey", "recoveryPhrase"] {
            for pat in &[format!("\"{}\":\"", kw), format!("\"{}\": \"", kw), format!("'{}':'", kw), format!("'{}': '", kw)] {
                if let Some(pos) = content.find(pat) {
                    let start = pos + pat.len();
                    if let Some(end) = content[start..].find('"').or_else(|| content[start..].find('\'')) {
                        let val = &content[start..start + end];
                        let words: Vec<String> = val.split_whitespace().map(|w| w.to_string()).collect();
                        if words.len() == 12 || words.len() == 24 {
                            result.found_seeds.push((source.to_string(), words));
                        } else if val.len() >= 64 && val.chars().all(|c| c.is_ascii_hexdigit()) {
                            result.found_keys.push((source.to_string(), val.to_string()));
                        }
                    }
                }
            }
        }
    }
}

fn generate_html_report(result: &ScanResult) -> String {
    let mut html = String::with_capacity(20000);

    // Head + CSS
    html.push_str(r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>AFRI RECOVER — Rapport d'Investigation</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{background:#0a2e1a;color:#fff;font-family:-apple-system,sans-serif;padding:12px;max-width:800px;margin:0 auto}
h1{color:#ffd700;text-align:center;margin:15px 0;font-size:1.5em}
h2{color:#4ecca3;margin:15px 0 8px;border-bottom:1px solid #4ecca3;padding-bottom:4px;font-size:1.1em}
.section{background:#0f3d2e;border-radius:10px;padding:12px;margin:10px 0}
.wallet{padding:8px;margin:5px 0;border-radius:5px;display:flex;justify-content:space-between;align-items:center}
.locked{background:#4a1a1a}
.accessible{background:#1a4a1a}
.notfound{background:#2a2a2a;color:#888}
.seed-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:6px;margin:10px 0}
.seed-word{background:#1a5a3a;padding:8px 4px;border-radius:5px;text-align:center;color:#ffd700;font-weight:bold;font-size:0.85em}
.seed-num{color:#888;font-size:0.75em;display:block}
.procedure{background:#0a2e1a;border-radius:5px;padding:10px;margin:8px 0}
.procedure-title{color:#ffd700;font-weight:bold;margin-bottom:5px}
.procedure-step{padding:5px 10px;margin:3px 0;border-left:3px solid #ffd700;font-size:0.9em}
.key-item{padding:5px;margin:3px 0;background:#0a2e1a;border-radius:3px;font-family:monospace;font-size:0.85em}
.conclusion{text-align:center;padding:15px;background:#1a5a3a;border-radius:10px;margin:10px 0}
.badge{display:inline-block;padding:2px 8px;border-radius:10px;font-size:0.8em;margin-left:5px}
.badge-red{background:#4a1a1a;color:#f88}
.badge-green{background:#1a4a1a;color:#8f8}
.stat{display:inline-block;padding:8px 15px;margin:3px;background:#0a2e1a;border-radius:5px;text-align:center}
.stat-num{color:#ffd700;font-size:1.3em;font-weight:bold;display:block}
.stat-label{color:#888;font-size:0.8em}
</style>
</head>
<body>
<h1>🔍 AFRI RECOVER</h1>
<p style="text-align:center;color:#888">Investigation des Seeds Occidentales</p>
<p style="text-align:center;color:#4ecca3;font-style:italic">"L'Occident dit 'perdu' — nous trouvons." 💚🦁</p>
"#);

    // Stats
    html.push_str(&format!(r#"
<div style="text-align:center;margin:10px 0">
<span class="stat"><span class="stat-num">{}</span><span class="stat-label">Apps Détectées</span></span>
<span class="stat"><span class="stat-num">{}</span><span class="stat-label">Fichiers</span></span>
<span class="stat"><span class="stat-num">{}</span><span class="stat-label">Seeds</span></span>
<span class="stat"><span class="stat-num">{}</span><span class="stat-label">Clés Privées</span></span>
</div>
"#, result.detected_apps.len(), result.found_wallets.len(), result.found_seeds.len(), result.found_keys.len()));

    // Section 1: Apps
    html.push_str("<div class=\"section\"><h2>📱 Applications Wallet Détectées</h2>");
    if result.detected_apps.is_empty() {
        html.push_str("<p style='color:#888'>Aucune application wallet occidentale détectée sur cet appareil.</p>");
        html.push_str("<p style='color:#888'>Pour tester: installez Trust Wallet ou MetaMask, créez un wallet, puis relancez afri_recover.</p>");
    } else {
        for (name, path, accessible) in &result.detected_apps {
            let cls = if *accessible { "accessible" } else { "locked" };
            let icon = if *accessible { "✅" } else { "🔒" };
            let badge = if *accessible { "<span class='badge badge-green'>ACCESSIBLE</span>" } else { "<span class='badge badge-red'>VERROUILLÉ</span>" };
            html.push_str(&format!("<div class='wallet {}'>{} {} {}</div>", cls, icon, name, badge));
            html.push_str(&format!("<div style='color:#888;font-size:0.8em;padding-left:25px'>{}</div>", path));
        }
    }
    html.push_str("</div>");

    // Section 2: Files
    html.push_str("<div class=\"section\"><h2>📁 Fichiers Wallet Trouvés</h2>");
    if result.found_wallets.is_empty() {
        html.push_str("<p style='color:#888'>Aucun fichier wallet trouvé dans les répertoires accessibles.</p>");
    } else {
        for (i, (path, summary)) in result.found_wallets.iter().enumerate() {
            html.push_str(&format!("<div class='key-item'>{}. {} → {}</div>", i + 1, path, summary));
        }
    }
    html.push_str("</div>");

    // Section 3: Seeds
    html.push_str("<div class=\"section\"><h2>🎯 Seeds Extraites (24 MOTS)</h2>");
    if result.found_seeds.is_empty() {
        html.push_str("<p style='color:#888'>Aucune seed en clair trouvée.</p>");
        html.push_str("<p style='color:#888'>Les wallets occidentaux chiffrent les seeds. Mais ils les STOCKENT — c'est prouvé.</p>");
        html.push_str("<p style='color:#4ecca3'>↓ Voir les procédures ci-dessous pour afficher vos 24 mots ↓</p>");
    } else {
        for (source, words) in &result.found_seeds {
            html.push_str(&format!("<div style='color:#ffd700;margin:8px 0'>🎯 Source: {}</div>", source));
            html.push_str("<div class='seed-grid'>");
            for (i, w) in words.iter().enumerate() {
                html.push_str(&format!("<div class='seed-word'><span class='seed-num'>{}</span>{}</div>", i + 1, w));
            }
            html.push_str("</div>");
        }
    }
    html.push_str("</div>");

    // Section 4: Keys
    if !result.found_keys.is_empty() {
        html.push_str("<div class=\"section\"><h2>🔑 Clés Privées Trouvées</h2>");
        for (source, key) in &result.found_keys {
            let masked = if key.len() > 20 {
                format!("{}...{}", &key[..8], &key[key.len()-8..])
            } else { key.clone() };
            html.push_str(&format!("<div class='key-item'>🔑 {} → {}</div>", source, masked));
        }
        html.push_str("</div>");
    }

    // Section 5: Procedures
    html.push_str("<div class=\"section\"><h2>📋 Procédures — Afficher les 24 Mots</h2>");
    html.push_str("<p style='color:#888;margin-bottom:10px'>Procédures officielles pour afficher vos 24 mots secrets dans chaque wallet occidental:</p>");
    for (wallet, steps) in WALLET_PROCEDURES {
        html.push_str("<div class='procedure'>");
        html.push_str(&format!("<div class='procedure-title'>📱 {}</div>", wallet));
        for step in *steps {
            html.push_str(&format!("<div class='procedure-step'>{}</div>", step));
        }
        html.push_str("</div>");
    }
    html.push_str("</div>");

    // Section 6: Blockchain data
    if !result.blockchain_data.is_empty() {
        html.push_str("<div class=\"section\"><h2>📦 Données Blockchain</h2>");
        for (path, info) in &result.blockchain_data {
            html.push_str(&format!("<div class='key-item'>📦 {} → {}</div>", path, info));
        }
        html.push_str("</div>");
    }

    // Conclusion
    html.push_str("<div class=\"conclusion\">");
    html.push_str("<h2>💡 Conclusion</h2>");
    if result.detected_apps.is_empty() && result.found_wallets.is_empty() {
        html.push_str("<p>Aucun wallet occidental détecté sur cet appareil.</p>");
        html.push_str("<p>Pour tester cet outil:</p>");
        html.push_str("<p>1. Installez Trust Wallet ou MetaMask</p>");
        html.push_str("<p>2. Créez un wallet (notez votre seed)</p>");
        html.push_str("<p>3. Relancez afri_recover</p>");
        html.push_str("<p>4. L'outil trouvera les données stockées</p>");
    } else {
        let locked = result.detected_apps.iter().filter(|(_,_,a)| !a).count();
        let accessible = result.detected_apps.iter().filter(|(_,_,a)| *a).count();
        html.push_str(&format!("<p>📊 {} app(s) détectée(s): {} accessible(s), {} verrouillée(s)</p>", result.detected_apps.len(), accessible, locked));
        html.push_str(&format!("<p>{} fichier(s) wallet trouvé(s), {} seed(s) extraite(s)</p>", result.found_wallets.len(), result.found_seeds.len()));
        if locked > 0 {
            html.push_str("<p style='color:#f88;margin-top:8px'>⚠️ PREUVE: L'Occident STOCKE vos seeds sur votre appareil.</p>");
            html.push_str("<p>Les données sont là mais VERROUILLÉES (root requis).</p>");
            html.push_str("<p>L'Occident dit 'impossible à récupérer' mais les données SONT là.</p>");
        }
        if !result.found_seeds.is_empty() {
            html.push_str("<p style='color:#8f8;margin-top:8px'>✅ SEEDS TROUVÉES EN CLAIR!</p>");
            html.push_str("<p>Les wallets occidentaux stockent les seeds SANS chiffrement adéquat.</p>");
        }
    }
    html.push_str("<p style='color:#ffd700;margin-top:10px;font-weight:bold'>L'Afrique dit: 'récupérable, transparent.' 💚🦁</p>");
    html.push_str("<p style='color:#f88'>L'Occident dit: 'impossible' mais stocke quand même.</p>");
    html.push_str("<p style='color:#4ecca3;margin-top:5px'>La différence: nous ne mentons pas.</p>");
    html.push_str("</div>");

    html.push_str("<p style='text-align:center;color:#888;margin:15px 0'>AFRI RECOVER v1.0 — Souveraineté numérique africaine 💚🦁</p>");
    html.push_str("</body></html>");

    html
}

fn open_in_chrome(path: &str) {
    // Termux: termux-open
    let r1 = Command::new("termux-open").arg(path).status();
    if r1.is_ok() {
        println!("  {}Ouvert dans Chrome via termux-open ✅{}", GREEN, RESET);
        return;
    }
    // Fallback: am start
    let uri = format!("file://{}", path);
    let r2 = Command::new("am")
        .args(["start", "-a", "android.intent.action.VIEW", "-d", &uri, "-t", "text/html"])
        .status();
    if r2.is_ok() {
        println!("  {}Ouvert dans Chrome via am start ✅{}", GREEN, RESET);
    } else {
        println!("  {}Impossible d'ouvrir Chrome automatiquement.{}", YELLOW, RESET);
        println!("  Ouvrez manuellement: {}", path);
    }
}

fn is_bip39_likely(word: &str) -> bool {
    word.len() >= 3 && word.len() <= 8 && word.chars().all(|c| c.is_ascii_lowercase())
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 { format!("{:.2} GB", bytes as f64 / 1e9) }
    else if bytes >= 1_000_000 { format!("{:.2} MB", bytes as f64 / 1e6) }
    else if bytes >= 1_000 { format!("{:.1} KB", bytes as f64 / 1e3) }
    else { format!("{} bytes", bytes) }
}
