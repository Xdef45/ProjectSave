# Général
Ce cas concerne toutes les requêtes qui demande d'être authentifié au préalable

## Output erreur
```
Erreur inconnue middleware : 0
```
```
Error variable app_data Auth inexistante: 1
```
```
Pas de cookie Bearer: 2
```
```
Token expiré: 3
```


# /api/signup
Lors de l'inscription d'un nouveau utilisateur, celui-ci lui envoie son username et password, il vérifie si l'utilisateur n'est pas déjà enregistré, l'ajoute à la base de données et lui renvoie un cookie d'authentification'.
## input
Type: ```application/json```
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


# /api/signin
Lors de la connection d'un utilisateur, celui-ci lui envoie son username et password, vérifie s'il est déjà enregistrer et lui renvoie son cookie d'authentification
## input
Type: ```application/json```
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
## output
Type: ```multipart/form_data```
```
<repot_key_encrypted>
```
# /api/send_ssh_key
Une fois l'utilisateur authentifier avec son cookie, il nous envoie sa clé ssh publique sous forme d'un fichier,on lui renvoie un status OK.
## input
Type: ```Header```
```
Cookie Bearer=<JWT_Token>
```
Type: ```multipart/form_data```
```
<ssh.pub_key>
```
## output
```
http code 200
```

