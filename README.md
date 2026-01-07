# StrongHolder
<img src="/documentations des outils/images/logo.png" width="30%">

## Sommaire 
1. **[Équipe](#1-équipe)**
2. **[Description du projet](#2-description-du-projet)**
3. **[Infrastructure](#3-infrastructure)**


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

Ce projet consiste en une solution de sauvegarde de fichiers sécurisée. Un utilisateur peut depuis une interface web/application sauvegarder ses fichiers en toute sécurité de n'importe où. Chaque utilisateur possède donc un compte qui lui permet d'intéragir avec ses sauvegardes et d'en créer des nouvelles. 

## 3. Infrastructure
L'infrastructure fonctionne en plusieurs étages :
![etage infra](/documentations%20des%20outils/images/étage%20infra.png)
L'hyperviseur principal va lui hyperviser 3 machines : 
- Le premier proxmox sur lequel nous mettrons toutes les machines nécessaires pour le reste de l'infrastructure
- Le deuxième proxmox est là pour simuler une redondance (N'ayant pas les moyens nécessaure pour acheter deux serveur pour faire une véritable infra c'est comme ça que nous faisons pour s'en rapprocher le plus possible)
- Le pfsense qui nous permet de configurer les différentes règles de pare-feu et de redirection de port