# API Sauvegarde
L'API Rest nous sert lors de la restauration des sauvegardes. Elle permet de requerire 

# Build
Afin d'accélerer
cargo chef cook
cache dock du build des dépendance gain de temps
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

## openssh
Les command sont préparer et les variables avec le "$" ne sont pas résolue, connection unique pour tous les commands
## sqlx
commande préparé connection unique pour toutes les requetes.
## argon2id
Paramètre
MEMORY_COST: u32 = 64*1024;
ITERATION_COST: u32 = 3;
PARALLELISM_COST: u32 = 4;
HASH_LENGTH: usize = 32;
Temps : 200ms