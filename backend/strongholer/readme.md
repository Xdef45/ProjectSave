# API Sauvegarde
# Installation de cargo
```curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh```

# Compilation
```cargo build```

# Execution
Après le phase de compilation, le binaire est créer dans target/debug/strongholder. Mais avant de l'exécuter, il lui faut un fichier ".env.json" dans le répertoire ou s'exercute le binaire. Il doit contenir ces paramètres :
### .env.json
```
{
    "db_host":"<HOST>",
    "db_port":"<PORT>",
    "db_user":"<USERNAME>",
    "db_password":"<PASSWORD>"",
    "db":"<DATABASE_NAME>"",
    "JWT_secret": "<secret>"
}
```
Puis ```./strongholder```