# Backend Strongholder
Afin que notre solution de sauvegarde soit opérationnelle, nous utilisons 4 dockers.
## Nginx
L’application qui est exécuté est nginx qui est un service http

Ce docker est lancé depuis l’image nginx:1.25-alpine 

Il sert de frontend pour à peu près toutes les fonctionnalités de sauvegarde. C’est lui qui va rediriger les requêtes des utilisateurs depuis https://strongholder.fr/api vers l’api. le sous domaine /api possède toutes les fonctionnalités telle que /api/create_user etc, etc. 
Le build de ce docker va lier les logs de nginx vers un dossier de la machine hôte pour avoir un meilleur contrôle des logs. Il va copier les fichiers de configurations et les fichiers web vers les bons dossiers pour que tout fonctionne.

Un script en js a été créé pour qu’il récupère les fichiers .exe et .deb de la release de notre github et qu’il serve de lien de téléchargement.

docker va lier les ports 443 de l’hote et de la machine pour la redirection vers le docker
## Borg
L’application qui est exécuté est openssh qui est un service ssh

Le docker de sauvegarde `strongholder-borg` est lancé sur une image `debian:latest` .

Le docker fait principalement tourner le service `openssh-server`, il expose son port 22 sur le pour 2222 de l'hôte, pour permettre à l’api ainsi qu’aux clients de faire des opérations par ssh.
## API
L’application qui est exécuté est strongholer qui est un service http

Ce docker est lancé depuis une image ```rust:alpine3.23``` à la compilation et dans ```alpine:3.23``` à l'exécution.

L’API est intégrer afin que lors de la restauration d’une sauvegarde chiffrer pour que cette action ne soit pas automatique comme pour la sauvegarde mais requiert des identifiants. Elle servira aussi à avoir un retour sur les logs, les archives et les fichiers qui ont été sauvegarder mais également à créer le dépôt Borg et authentifier les utilisateurs. Elle est codée avec le framework Actix_web basé sur Rust.
## Base de données
L’application exécuté est MariaDB qui est un service Mysql

Cette base données est une image `mariadb:12.2-noble` est sert à l’api pour stocker les utilisateurs enregistrés. Je lui donne juste sont `.env` avec les identifiants et le nom de la base données à créer. En complément, je lui donne un export de la structure de la base de données à créer.
# Processus de déploiement
## Création des secrets
Un docker-compose ```gen_credentials``` a été réalisé. À son lancement, il génère des secret pour les 4 autres docker. Pour y parvenir, il utilise un volume persistant pour pouvoir exporter ses secrets. Son image est un alpine:3.14 avec openssl et openssh d’installés.

```
docker compose -f credentials/docker-compose.yml up --build 
```
## Définir l'emplacement des volumes partagés
Étant donné que nos docker stocke les sauvegarde, les clés et logs, nous avons créer des volumes persistants. Pour choisir où les stockées une variable ```VOLUMES_PATH``` est à définir dans un ```.env``` de ce dossier.

## Exécution des docker
Enfin pour lancer la solution et démarrer les 4 docker, voici la commande à exécuter.
```
docker compose up --build
```
Après quoi le port 2222 et 443 seront exposés sur la machine hôte.