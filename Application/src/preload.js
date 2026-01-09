const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('api', {
    // Fonction pour demander le contenu d'un dossier
    readDir: (path) => ipcRenderer.invoke('read-directory', path),
    // Fonction pour récupérer le séparateur de chemin ( \ ou / ) selon l'OS
    pathSep: process.platform === 'win32' ? '\\' : '/',

    // Le frontend appellera window.api.genKey(...)
    genKey: (clientName) => ipcRenderer.invoke('run-genkey', clientName),

    // Le frontend appellera window.api.backup(...)
    backup: (clientName, path) => ipcRenderer.invoke('run-backup', clientName, path)
});