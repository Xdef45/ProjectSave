## Uuuh ptite explications sur les scripts Linux client

On rentre dans le folder avec tout les scripts  

1. **Installation des scripts**  
```sudo ./install_all_client.sh```

2. something will be here
    ```
    sudo tee /home/borghelper/.ssh/authorized_keys >/dev/null <<'EOF'
    command="/usr/local/sbin/borghelper_dispatch.sh",no-pty,no-agent-forwarding,no-X11-forwarding,no-port-forwarding <server_to_client_pubkey> server_to_client
    EOF
    sudo chown borghelper:borghelper /home/borghelper/.ssh/authorized_keys
    sudo chmod 600 /home/borghelper/.ssh/authorized_keys
    ```

3. **Génération des clés client**  
```sudo /usr/local/sbin/client_genkey.sh <client_name>```

4. **Envoi des clés publiques à l'api**

5. **Récupération de la clé .gpg**  
Pour l'instant j'ai ça (test)  
```sudo /usr/local/sbin/client_fetch_bootstrap_key.sh```  

6. **Lancer la save**  
```/usr/local/sbin/client_backup.sh <client> <path_to_save>```

Features manquantes qui seront possibles avec l'api
- Listing des fichiers dans les archives
- restauration des saves