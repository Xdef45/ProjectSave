# API Sauvegarde
# API

L’application qui est exécuté est strongholer qui est un service http

Le Rust étant un langage compiler à besoin après chaque changement de code de se recompiler ce qui prends énormément de temps. Pour réduire se temps qui peux aller jusqu’à 5 minutes, on compile sur un stage docker uniquement les dépendance, c’est ce qui prends le plus de temps. Pour ce faire on utilise le crate chef qui extraire à partir du projet un json qui contient toute les dépendances présentent dans le fichier Cargo.toml. 

## Chef

Le premier stage est l’installation des packets nécessaire à compiler le projet, à partir d’un image docker `rust:alpine3.23` pour avoir le moins de chose installer. Le packet openssl est nécessaire puisque c’est ce qu’on utilise dans notre API pour le chiffrement et la dérivation de clé, puis le reste c’est pour build aussi le code C de openssl avec l’API. Mais aussi cargo-chef qui nous servira à extraire les dépendance

![image.png](../../documentations%20des%20outils/images/API/chef.png)

## Planner

Puis on créer le stage qui extraire les dépendances

![image.png](../../documentations%20des%20outils/images/API/planner.png)

## Builder

Ensuite on build les dépendance et l’application en deux layers différent. Le premier stage n’est jamais modifié donc reste en cache(très rapide). Le deuxième stage change dès que le code est modifier mais tout ce qu’il fait il créer un fichier json à partir du Cargo.toml(rapide).Donc si le Cargo.toml n’est pas modifé le résultat du recipe.json reste inchangé. 

C’est pour ça qu’au stage `builder` , la compilation des dépendance se réalise uniquement si le fichier recipe.json donné par le stage `planner` est modifié. Et si ce n’est pas le cas il est gardé en cache (rapide sans changement des dépendance sinon prends 4 minutes de compilation).

![image.png](../../documentations%20des%20outils/images/API/Builder.png)

Puis on compile le reste du code source 

![image.png](../../documentations%20des%20outils/images/API/Builder2.png)

## Runtime

Puis on exécute l’applicaiton copier à partir du stage builder.

![image.png](../../documentations%20des%20outils/images/API/runtime.png)

# Dependance
- actix-web = {version = "4.12.1", features = ["cookies"]}
- bytes = "1.11.1"
- derive_more = "2.1.1"
- futures-core = "0.3.31"
- hex = "0.4"
- openssh = { version = "0.11.6", features = ["native-mux"] }
- openssh-sftp-client = { version = "0.15.4", features = ["openssh"] }
- passcheck = "0.2.0"
- serde = "1.0.228"
- serde_json = "1.0.149"
- tokio = { version = "1.49.0", features = ["fs", "io-util", "macros", "process", "rt-multi-thread"] }
- jsonwebtoken = {version = "10.2.0", features = ["aws_lc_rs"]}
- openssl = {version = "0.10", features = ["vendored"]}
- sqlx = {version = "0.8.6", features = ["runtime-async-std", "mysql"]}
- uuid = {version = "1.19.0", features = ["v4"]}

## OpenSSH

Grâce au crate openssh, la même connexion est utilisé pour toute les commandes SSH executé sur le serveur Borg. Il .utilise en complément shell-escape qui exfiltre ou parse quand il reçoit en argument $ ou un argument qui à des espaces. 

## SQLX

Sqlx utilise également un seul session pour toutes c’est requête et un utilise les requête préparer pour éviter les injections SQL.

Mais étant donnée que Arctix est basé sur de l’asynchrone il faut pouvoir passer la variable qui contient la session SSH et SQL aux threads créer par les multiple requête HTTP. Pour ce faire, ont utilise std::sync::Arc afin créer un pointeur intelligent. Ce pointeur doit être créer avant le démarrage de Arctix et être partager par tous les theards. Arctix permet de partager une instance avec la methode add_data() et être récupérer lors de la requête avec web::Data(T) et le type de l’instance.
## OpenSSL
### Argon2id
Paramètre
- MEMORY_COST: u32 = 64*1024;
- ITERATION_COST: u32 = 3;
- PARALLELISM_COST: u32 = 4;
- HASH_LENGTH: usize = 32;

Temps : 150ms