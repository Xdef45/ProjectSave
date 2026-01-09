# Authentification
## signup
À partir de son username et password, on rechercher si l'utilisateur n'est pas déjà inscrit. Sinon, on lui créer un uuid aléatoire, on créer sa clé dériver avec son mot de passe et lui créer sa clé aes pour borg. Cette clé borg est chiffrer avec sa clé dériver convertit en hexadécimal. Puis son uuid, son username et sa clé aes chiffrer son ajouter à la base de données. Pour finir, on lui renvoie dans un cookie JWT son uuid et sa clé dériver.
## signin
À partir de son username et password, on rechercher si l'utilisateur n'est pas déjà inscrit. Si oui, on créer sa clé dériver avec son mot de passe et tentons de déchiffrer sa clé aes en base de données. En cas de réussite, on lui renvoie dans un cookie JWT son uuid et sa clé dériver.
![Texte alternatif](../../base_de_donnees.png)
## Création d'un token JWT
Le token est son forme d'un JSON qui contient une date d'expiration (```exp```), l'id du client (```id```) et son mot de passe dérivé (```kdf```) servira à déchiffrer son répôt Borg
```
{
    exp: u64,
    id: String,
    kdf: String
}
```
## Extraire la clé dériver
À partir de son mot de passe, on utilise argon2id avec pour paramètre:
- Espace mémoire: 64MB
- Nombre d'itération: 3
- Nombre de paraléllisme: 4
- Longueur du hash: 32
## Vérification de la validité du token JWT
Lors de sa création, ce token révoquer après 10min d'inactivité. 5min avant son expiration, il est rafraichit. 
