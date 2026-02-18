# Général
Ce cas concerne toutes les requêtes qui demande d'être authentifié au préalable

## Output erreur
```
APIError::NoFile=>"600",
            APIError::Metadata=>"601",

            // Cas Généraux
            APIError::NoCookieBearer=>"101",
            APIError::NoAuthAppData=>"102",
            APIError::Script=>"103",
            APIError::Ssh=>"104",
            APIError::Sftp=>"105",
            APIError::ValidInput=>"106",

            // File
            APIError::Write=>"200",

            //Convertion
            APIError::UTF8=>"300",
            APIError::Json=>"301",
            APIError::Usize=>"302",

            //Logup
            APIError::AlreadyExist=>"1",
            APIError::UsernameTooShort=>"2",
            APIError::InvalidPassword=>"3",
            APIError::PasswordTooShort=>"4",
            APIError::SpecialCharMissing=>"5",
            APIError::MajusculeMissing=>"6",
            APIError::NumberMissing=>"7",

            // Login
            APIError::NotSignup=>"0",

            //Bearer
            APIError::Expired=>"Effacement du cookie"
            APIError::ErrorBearer=>"504",

            // token
            APIError::EncodeToken=>"700",
            //Encryption
            APIError::KDFError =>"400"
```

# /api/signup
Lors de l'inscription d'un nouveau utilisateur, celui-ci lui envoie son username et password, il vérifie si l'utilisateur n'est pas déjà enregistré, l'ajoute à la base de données et lui renvoie un cookie d'authentification'.
## input
Type: ```application/json``` | method: ```post```
```
{
    "username": "marc-antoine.dumar@gmail.com",
    "password": "Tetris123@"
}
```
## output
```L'utilisateur existe déjà
Set-Cookie Bearer=<JWT_Token>
```
## output erreur
``` 
Erreur inconnue: 0
```
``` 
L'utilisateur existe déjà: 1
```
``` 
Username minimum 5: 2
```
``` 
Erreur dans le mot de passe inconnue: 3
```
``` 
Mot de passe inférieur à 12 dans le mot passe: 4
```
``` 
Pas de caractère spéclial dans le mot passe: 5
```
``` 
Pas de majuscule dans le mot passe: 6
```
``` 
Pas de nombre dans le mot passe: 7
```
``` 
Error lors de la création du KDF: 8
```


# /api/signin
Lors de la connection d'un utilisateur, celui-ci lui envoie son username et password, vérifie s'il est déjà enregistrer et lui renvoie son cookie d'authentification
## input
Type: ```application/json``` | method: ```post```
```
{
    "username": "marc-antoine.dumar@gmail.com",
    "password": "Tetris123@"
}
```
## output
```
Set-Cookie Bearer=<JWT_Token>
```
## output erreur
```
Erreur inconnue : 0
```
```
L'utilisateur n'est pas enregistré: 1
```

# /api/get_repot_key
Une fois l'utilisateur authentifier avec son cookie, on lui envoie sous forme de fichier téléchargeable sa clé master 1.
## input
```
Cookie Bearer=<JWT_Token>
```
## Output
Type: ```application/octet stream```
```
<repot_key_encrypted>
```
## Output erreur
```
Fichier pas trouvé: 1
```
```
Erreur inconnue: 0
```

# /api/get_ssh_pub_key_server
Envoie à l'utilisateur de la clé ssh publique du serveur pour qu'il se connecte à l'utilisateur
## input
Type: ```application/json``` | method: ```post```


# /api/send_ssh_key_tunnel
L'utilisateur envoie au serveur la clé ssh publique pour se connecter à l'utilisateur tunnel sur le serveur
## input
Type: ```application/json``` | method: ```post```
```
{
    ssh_key: <ssh_key_value>
}
```

# /api/send_ssh_key
Une fois l'utilisateur authentifier avec son cookie, il nous envoie sa clé ssh publique sous forme d'un fichier,on lui renvoie un status OK.
## input
### Header
```
Cookie Bearer=<JWT_Token>
```
Type: ```application/json```
```
{
    ssh_key: <ssh_key_value>
}
```
## output
```
http code 200
```

# /api/get_list
Une fois l'utilisateur authentifier avec son cookie, il demande le contenue de repot Borg sous forme d'un json.
## input
Type: ```Header```
```
Cookie Bearer=<JWT_Token>
```
body vide pour lister les archives disponibles ou
```
{
    archive_name: <archive_name>
}
```
pour lister le contenu des archive
## output
Type: ```application/json```
```
{
    "archives": []
}
```
ou avec archive_name spécifier
```
{
    "archive_name": "<nom_de_l'archive>"
    "archive_content": [
        {
            "type": "-", 
            "path": "mnt/c/Users/arthu/Documents/Analyse Fonctionnelle/BeteACorne.png", 
            "mtime": "2025-11-07T13:51:02.354852", "size": 108736
        },
        {
            "type": "-", 
            "path": "mnt/c/Users/arthu/Documents/Analyse Fonctionnelle/BeteACorne.png", 
            "mtime": "2025-11-07T13:51:02.354852", "size": 108736
        }
    ]
}
```
# /api/get_restore
## input
Type: ```Header```
```
Cookie Bearer=<JWT_Token>
```
```
{
    archive_name: <archive_name>
}
```
## output
Type: ```application/octet-stream```
```
<archive>.tar.gz
```

# api/get_log
## input
### Header
```
Cookie Bearer=<JWT_Token>
```
## output
