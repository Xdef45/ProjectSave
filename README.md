# StrongHolder
<img src="/documentations des outils/images/logo.png" width="30%">

## Sommaire 
1. **[Équipe](#1-équipe)**
2. **[Description du projet](#2-description-du-projet)**
3. **[Infrastructure](#3-infrastructure)**
4. **[Fonctionnalités](#4-fonctionnalités)**
5. **[Technologies utilisées](#5-technologies-utilisées)**
6. **[Fonctionnement](#6-fonctionnement)**

## 1. Équipe

| **Membres** | **Rôles** |
| --- | --- |
| Samuel Bredas | Dev scripts |
| Clément | infra |
| Arthur | Dev Application |
| Samuel Ardilla | Dev Application |
| Marc Antoine | Administration Système |

## 2. Description du projet
![Architecture-projet](/documentations%20des%20outils/images/archi.png)

Ce projet consiste en une solution de sauvegarde de fichiers sécurisée. Un utilisateur peut depuis une interface web/application sauvegarder ses fichiers en toute sécurité de n'importe où. Chaque utilisateur possède donc un compte qui lui permet d'interagir avec ses sauvegardes et d'en créer des nouvelles. 

## 3. Infrastructure
L'infrastructure fonctionne en plusieurs étages :
![etage infra](/documentations%20des%20outils/images/étage%20infra.png)
L'hyperviseur principal va hyperviser 3 machines : 
- Le premier Proxmox sur lequel nous mettrons toutes les machines nécessaires pour le reste de l'infrastructure
- Le deuxième Proxmox est là pour simuler une redondance (N'ayant pas les moyens nécessaires pour acheter deux serveurs pour faire une véritable infrastructure c'est comme ça que nous faisons pour s'en rapprocher le plus possible)
- Le pfSense qui nous permet de configurer les différentes règles de pare-feu et de redirection de port

## 4. Fonctionnalités
Voici une liste exhaustive des fonctionnalités disponibles dans notre projet:
- Sauvegarde de fichiers chiffrés
- Gestionnaire de mots de passe
- Interface web
- Application Electron

## 5. Technologies utilisées
* Comme hyperviseur principal nous avons choisi [Proxmox](https://www.proxmox.com/en/) car c'est un outil très complet nous donnant toutes les fonctionnalités nécessaires pour notre projet.
* Comme routeur/pare-feu nous utilisons [pfSense](https://www.pfsense.org/), outil très puissant et gratuit nous donnant tous les outils nécessaires concernant les accès aux différentes machines, redirection de port et la création des VLANs pour isoler les machines entre elles et n'autoriser que le trafic nécessaire.
* L'outil de sauvegarde que nous utilisons est [Borg](https://www.borgbackup.org/) car il répond parfaitement à la demande des sauvegardes chiffrées et qu'il est open source.
* Pour la sécurité de toutes les machines de l'infrastructure nous utilisons la solution open source [Wazuh](https://wazuh.com/) qui nous permet d'avoir une vue d'ensemble sur les machines et des vulnérabilités.
* Comme gestionnaire de mots de passe intégré, nous avons choisi la solution [VaultWarden](https://github.com/dani-garcia/vaultwarden) qui est gratuite et open source. On ajoute à cela la solution [Zabbix](https://www.zabbix.com/fr) qui nous permet une vue d'ensemble des machines concernant leurs statistiques CPU, mémoire et leur stockage.
* Les sauvegardes sont sauvegardées sur des machines utilisant le système de [Ceph](https://ceph.io/en/), qui réplique automatiquement les sauvegardes sur d'autres disques pour garantir une redondance.

## 6. Fonctionnement 

1. Un utilisateur crée un compte sur le site avec un nom d'utilisateur et un mot de passe 
2. Il peut alors télécharger notre application
![ScreenApp](/documentations%20des%20outils/images/AppAccueil.png)
3. Il peut désormais envoyer ses sauvegardes en tout sécurité sur notre infrastructure