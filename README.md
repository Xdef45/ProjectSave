# StrongHolder
<img src="/documentations des outils/images/logo.png" width="30%">

## Sommaire 
1. **[Équipe](#1-équipe)**
2. **[Description du projet](#2-description-du-projet)**
3. **[Infrastructure](#3-infrastructure)**
4. **[Fonctionnalités](#4-fonctionnalités)**
5. **[Technnologies utilisées](#5-technologies-utilisées)**
6. **[fonctionneent](#6-fonctionnement)**

## 1. Équipe

| **Membres** | **Rôles** |
| --- | --- |
| Samuel Bredas | Dev scripts |
| Clément | Administrateur Système et réseau |
| Arthur | Dev Application |
| Samuel Ardilla | Analyste Supervision |
| Marc Antoine | Administrateur Système et réseau |

## 2. Description du projet

Ce projet consiste en une solution de sauvegarde chiffrée de fichiers sécurisée. Un utilisateur peut depuis une interface web/application sauvegarder ses fichiers en toute sécurité de n'importe où. Chaque utilisateur possède donc un compte qui lui permet d'intéragir avec ses sauvegardes et d'en créer des nouvelles. 

## 3. Infrastructure
L'infrastructure fonctionne en plusieurs étages :
### Niveau 1
![etage infra](/documentations%20des%20outils/images/infra%20niveau%201.png)
Le premier niveau correspond à machine physique qui est un hyperviseur Proxmox hébergé chez l'un de nos collaborateurs du projet.

Elle est composé de:
- 20 coeurs
- 32Go ram
- 1,5To de stockage ssd
- 1,5To de stockage externe (sauvegarde)
- 2.5GbE Interface 

Il servira à virtualisé une infrastructure redondante. C'est pour nous mettre dans un environnement redondant sans dépense trop d'argent
### Niveau 2
![](/documentations%20des%20outils/images/infra%20niveau%202.png)
Il s'agit de l'infrastructure physique virtualisé par le Proxmox du niveau. Pour des soucis de confusion, nous allons considérer cette infrastructure virtuelle comme l'infrastructure physique du projet.

 Machine | OS | Nombre de coeurs | Quantité de ram | Stockage | Interface réseau 
---------|----|---------------|---------------| ---------|-----------------
Pfsense | Pfsense | 2 | 2Go | **D1:** 30Go 32Go | 3
Ceph | Ubuntu | 2 | 2Go | **D1:** 30Go ; **D2:** 250Go | 1
Proxmox | Proxmox | 10 | 10Go | **D1:** 100Go | 1
Supervision | Debian | 4 | 4Go | **D1:** 150Go | 1

### Niveau 3
![](/documentations%20des%20outils/images/infra%20niveau%203.png)
Voici l'infrastructure complète avec toutes le VM, service et conteneur disponible.

**Services Pfsense**
Service|Fonction|Description
---|---|---
Wireguard|VPN|Accès distant au réseau interne
Haproxy|Proxy|Le vault est accessible uniquement par le proxy et ajoute le chiffrement HTTP que vaultwarden n'a pas

**Machine virtuelle**

Machine virtuelle|Fonction|Desciption
---|---|---
Vault|Gestionnaire de mot de passe|Sont enregistrés tous les identifinats de donnexion aux machine des collaborateurs
Sauvegarde|Infrastructure de sauvegarde|Contient les docker chargés d'assurer le service de sauvegarde de strongholder

**Conteneur docker**

Service|Hôte|Fonction|Description
---|---|---|---
Wazuh|Supervision|SIEM|Centralisation des logs de toutes le machine et VM
Zabbix|Supervision|Monitoring|Centralisation de l'état de charge de toutes les machines et VM
Nginx|Sauvegarde|Proxy|Ajout du chiffrement HTTP pour l'API Rest et du filtrage des requête HTTP
Borg|Sauvegarde|Sauvegarde|Service SSH pour l'accès au client à leur repos Borg
API|Sauvegarde|API Rest|Service HTTP pour l'accès au client à leur repos Borg
MariaDB|Sauvegarde|Base de données|Enregistrement des identifiants des clients

### Réseau

Nom interface|Type réseau|Adresse réseau|Acteurs
---|---|---|---
WAN|Physique|192.168.1.0/24|Internet
Wireguard|P2P|10.0.0.0/24|Collaborateur distant
Virtualisation|Physique|10.0.10.0/24|Ceph, Proxmox
Supervision|Physique|10.0.21.0/24|Supervision
VVault|Vlan|10.0.20.0/24|Vault
VSauvegarde|Vlan|10.0.30.0/24|Sauvegarde
VTest|Vlan|10.0.40.0/24|Proxmox Test

## 4. Fonctionnalités
Voici une liste exhaustive des fonctionnalités disponibles dans notre projet:
- Sauvegarde de fichiers chiffré
- Gestionnaire de mot de passe
- Interface WEB
- Application electron

## 5. Technologies utilisées
* Comme hyperviseur principal nous avons choisi [Proxmox](https://www.proxmox.com/en/) car c'est un outil très complet nous donnant toutes les fonctionnalitées nécessaire pour notre projet.
* Comme routeur/pare-feu nous utilisons [Pfsense](https://www.pfsense.org/), outil très puissant et gratuit nous donnant tout les outils nécessaires concernant les accès aux différentes machines, redirection de port et la création des VLANs pour isoler les machines entre elles et n'autoriser que le traffic nécessaire.
* L'outil de sauvegarde que nous utilisons est [Borg](https://www.borgbackup.org/) car il répond parfaitement à la demande des sauvegardes chiffrées et qu'il est open source.
* Pour la sécurité de toutes les machines de l'infrastructure nous utilisons la solution open source [Wazuh](https://wazuh.com/) qui nous permet d'avoir une vue d'ensemble sur les machines et des vulnérabilités.
* Comme gestionnaire de mot de passe intégré, nous avons choisi la solution [VaultWarden](https://github.com/dani-garcia/vaultwarden) qui est gratuite et open source. On ajoute à cela la solution [zabbix](https://www.zabbix.com/fr) qui nous permet une vue d'ensemble des machines concernant leur statistiques CPU, mémoire et leur stockage.
* Les sauvegardes sont sauvegarder sur des machines utilisant le système de [Ceph](https://ceph.io/en/), qui réplique automatiquement les sauvegardes sur d'autres disques pour garantir une redondance.

## 6. Fonctionnement 

1. Un utilisateur se créer un compte sur le site avec un nom d'utilisateur et un mot de passe 
2. Il peut alors télécharger notre application
![ScreenApp](/documentations%20des%20outils/images/AppAccueil.png)
3. Il peut désormais envoyer ses sauvegardes en tout sécurité sur notre infrastructure