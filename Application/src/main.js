const { app, BrowserWindow, ipcMain, Menu } = require('electron');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { exec } = require('child_process');

Menu.setApplicationMenu(null);

const createWindow = () => {
    const win = new BrowserWindow({
        width: 800,
        height: 600,
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),

            contextIsolation: true,
            enableRemoteModule: false,
            nodeIntegration: false
        }
    })

    win.loadFile(path.join(__dirname, 'login.html'))
}

app.whenReady().then(() => {
    createWindow()

    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) {
            createWindow()
        }
    })
})

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        app.quit()
    }
})



// Fonction pour convertir "C:\Dossier" en "/mnt/c/Dossier"
function toWslPath(windowsPath) {
    if (!windowsPath) return '';
    // 1. Remplacer les backslashes \ par des slashes /
    let wslPath = windowsPath.replace(/\\/g, '/');

    // 2. Remplacer "C:" par "/mnt/c" (et gérer les minuscules/majuscules)
    // On capture la lettre du lecteur (ex: C, D, E)
    wslPath = wslPath.replace(/^([a-zA-Z]):/, (match, driveLetter) => {
        return `/mnt/${driveLetter.toLowerCase()}`;
    });

    return wslPath;
}

// --- FONCTION UTILITAIRE : Trouver le bon chemin du script local ---
function getLocalScriptPath(scriptName) {
    // Si l'app est packagée (installée), les scripts sont souvent mis à côté de l'executable ou dans resources
    // app.isPackaged est true en prod, false en dev
    if (app.isPackaged) {
        // En prod, on ira chercher dans le dossier resources (voir configuration builder plus bas)
        return path.join(process.resourcesPath, 'scripts', scriptName + '.sh');
    } else {
        // En dev, c'est juste à côté
        return path.join(__dirname, 'scripts', scriptName + '.sh');
    }
}



ipcMain.handle('read-directory', async (event, folderPath) => {
    let defaultPath = (process.platform === 'win32') ? 'C:\\' : '/';
    const targetPath = folderPath || defaultPath;

    try {
        const dirents = await fs.promises.readdir(targetPath, { withFileTypes: true });

        const files = dirents
            .map(dirent => ({
                name: dirent.name,
                isDirectory: dirent.isDirectory(),
                path: path.join(targetPath, dirent.name)
            }))
            .sort((a, b) => {
                if (a.isDirectory === b.isDirectory) return a.name.localeCompare(b.name);
                return a.isDirectory ? -1 : 1;
            });

        return { path: targetPath, files: files };
    } catch (err) {
        console.error("Erreur lecture dossier:", err);
        return { error: "Accès refusé ou dossier vide (Protection système)." };
    }
});


// 1. Fonction pour générer la clé
ipcMain.handle('run-genkey', async (event, clientName) => {
    return new Promise((resolve, reject) => {
        const safeClientName = clientName.replace(/[^a-zA-Z0-9_-]/g, '');
        const scriptPath = '/usr/local/sbin/client_genkey.sh';
        let command = '';

        if (os.platform() === 'win32') {
            command = `wsl sudo ${scriptPath} ${safeClientName}`;
        } else {
            command = `sudo ${scriptPath} ${safeClientName}`;
        }

        console.log(`[Main] Exécution : ${command}`);

        exec(command, (error, stdout, stderr) => {
            if (error) {
                console.error(`[Main] Erreur : ${error.message}`);
                // On rejette l'erreur pour que le frontend sache que ça a échoué
                reject(error.message);
            } else {
                console.log(`[Main] Succès : ${stdout}`);
                resolve(stdout);
            }
        });
    });
});

// 2. Fonction pour le backup
ipcMain.handle('run-backup', async (event, clientName, pathToSave) => {
    return new Promise((resolve, reject) => {
        const safeClientName = clientName.replace(/[^a-zA-Z0-9_-]/g, '');
        const scriptPath = '/usr/local/sbin/client_backup.sh';
        let command = '';

        if (os.platform() === 'win32') {
            // Conversion chemin Windows -> WSL (C:\User -> /mnt/c/User)
            const wslPath = pathToSave
                .replace(/^([a-zA-Z]):/, (match, drive) => `/mnt/${drive.toLowerCase()}`)
                .replace(/\\/g, '/');
            command = `wsl ${scriptPath} ${safeClientName} "${wslPath}"`;
        } else {
            command = `${scriptPath} ${safeClientName} "${pathToSave}"`;
        }

        console.log(`[Main] Exécution : ${command}`);

        exec(command, (error, stdout, stderr) => {
            if (error) {
                console.error(`[Main] Erreur : ${error.message}`);
                reject(error.message);
            } else {
                console.log(`[Main] Succès : ${stdout}`);
                resolve(stdout);
            }
        });
    });
});