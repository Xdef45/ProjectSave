# Documentation - Ceph Cluster (Lab Setup)

## 1. Présentation

**Ceph** est une plateforme de stockage distribuée open-source.  
Elle permet de gérer des volumes, des objets et des systèmes de fichiers de manière unifiée.  
Ceph repose sur trois composants principaux :

- **MON (Monitor)** : maintient la carte du cluster et la cohérence globale.
- **OSD (Object Storage Daemon)** : stocke les données sous forme d’objets.
- **MGR (Manager)** : fournit une interface de gestion et des services supplémentaires (dashboard, metrics, etc.).

---

## 2. Architecture de base

Un cluster Ceph classique comprend :

| Rôle | Description | Exemple de machine |
|------|--------------|--------------------|
| Moniteur (MON) | Gère l’état du cluster et la topologie | ceph (192.168.56.103) |
| Manager (MGR) | Fournit le dashboard et la gestion | ceph (192.168.56.103) |
| OSD | Stocke physiquement les données | ceph2, ceph3 |
| Client | Accède au cluster (via RBD, CephFS ou S3) | Machine de test, VM cliente |

---

## 3. Installation (avec Cephadm)

### 3.1 Prérequis
- OS : Ubuntu 22.04+ ou Debian 12+
- Accès root sur les VMs
- SSH sans mot de passe entre les nœuds

### 3.2 Installation de Cephadm
```bash
curl --silent --remote-name https://download.ceph.com/keys/release.asc
sudo apt install -y cephadm
```

### 3.3 Bootstrap du cluster
```bash
sudo cephadm bootstrap --mon-ip IP.DU.DASHBOARD
```

Cette commande déploie :
- un **moniteur** (MON)
- un **manager** (MGR)
- un **dashboard** accessible sur https://IP.DU.DASHBOARD:8443

---

## 4. Ajout des nœuds au cluster

### 4.1 Ajout d’un nouvel hôte
Sur le nœud principal :
```bash
sudo ceph orch host add ceph2 IP.MACHINE.2
sudo ceph orch host add ceph3 IP.MACHINE.3
```

### 4.2 Déploiement des OSDs
Lister les disques détectés :
```bash
sudo ceph orch device ls
```

Puis déployer :
```bash
sudo ceph orch daemon add osd ceph2:/dev/sdb
sudo ceph orch daemon add osd ceph3:/dev/sdb
```

---

## 5. Commandes utiles

| Action | Commande |
|--------|-----------|
| Voir l’état du cluster | `ceph -s` |
| Lister les daemons | `cephadm ls` |
| Lister les hôtes | `ceph orch host ls` |
| Lister les services | `ceph orch ps` |
| Redémarrer un service | `ceph orch restart <type>` |
| Accéder au shell Ceph | `sudo cephadm shell` |

---


## 7. Outils de gestion

- **Dashboard Web** : https://192.168.56.103:8443  
  (login par défaut : `admin`, mot de passe affiché après bootstrap)
- **Prometheus** : http://192.168.56.103:9095  
- **Grafana** : http://192.168.56.103:3000  

---

## 8. Exemple d’utilisation avec RBD

### Création d’un pool et d’une image RBD
```bash
ceph osd pool create mypool 64
rbd create mypool/disk1 --size 2G
```

### Mapping du volume sur une VM Linux
```bash
modprobe rbd
rbd map mypool/disk1 --name client.admin
mkfs.ext4 /dev/rbd0
mount /dev/rbd0 /mnt/rbdtest
```

---

## 9. Redémarrage et maintenance

| Action | Commande |
|--------|-----------|
| Redémarrer tous les daemons | `ceph orch restart` |
| Redémarrer uniquement les mons | `ceph orch restart mon` |
| Voir les logs | `cephadm logs --name mon.ceph` |
| Supprimer un hôte | `ceph orch host rm <nom>` |

---

## 10. Nettoyage du lab

Pour supprimer complètement le cluster :
```bash
sudo cephadm rm-cluster --fsid <FSID> --force
```

---

## 11. Points clés à retenir

- Le **MON** gère la carte du cluster et les métadonnées.  
- Les **OSD** stockent et répliquent les données.  
- Le **MGR** fournit la supervision et le dashboard.  
- La **clé admin** permet de contrôler le cluster depuis n’importe quelle machine.  
- La **haute disponibilité** repose sur la réplication automatique entre OSDs.  

---

## 12. Ressources utiles

- [Documentation officielle Ceph](https://docs.ceph.com/en/latest/)
- [Cephadm guide complet](https://docs.ceph.com/en/latest/cephadm/)
- [RBD (RADOS Block Device)](https://docs.ceph.com/en/latest/rbd/)
- [CephFS (Filesystem)](https://docs.ceph.com/en/latest/cephfs/)
