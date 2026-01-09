const { spawn } = require('child_process');
const os = require('os');
const path = require('path');
const fs = require('fs');

// --- CONFIGURATION ---
const TEMP_DIR = path.join(os.tmpdir(), 'strongholder-install-' + Date.now());

// Liste EXACTE de tes fichiers dans le dossier /scripts
const SCRIPT_FILES = [
    'prepclient.ps1',
    'prepclient.sh',
    'install_all_clients.sh',
    'client_genkey.sh',
    'client_backup.sh',
    'initial_connection.sh'
];

// --- UTILITAIRES ---
function runCommand(cmd, args, name) {
    return new Promise((resolve, reject) => {
        console.log(`🔹 [${name}]...`);
        const child = spawn(cmd, args, { stdio: 'inherit', shell: true });
        child.on('close', code => (code === 0 ? resolve() : reject(new Error(`${name} échoué (Code ${code})`))));
    });
}

// Convertit C:\Users... en /mnt/c/Users... pour WSL
function toWslPath(winPath) {
    return path.resolve(winPath)
        .replace(/^([a-zA-Z]):/, (_, d) => `/mnt/${d.toLowerCase()}`)
        .replace(/\\/g, '/');
}

// Copie récursive de dossier
function copyFolderSync(from, to) {
    if (!fs.existsSync(to)) fs.mkdirSync(to, { recursive: true });
    fs.readdirSync(from).forEach(element => {
        const src = path.join(from, element);
        const dest = path.join(to, element);
        if (fs.lstatSync(src).isDirectory()) copyFolderSync(src, dest);
        else fs.copyFileSync(src, dest);
    });
}

// --- ETAPE 1 : INSTALLATION DE L'APPLICATION ---
async function installApp() {
    console.log("\n📦 1. Installation de l'application (Program Files)...");
    const platform = os.platform();

    // Chemins
    let srcInExe = '';
    let destSystem = '';
    let exePath = '';

    if (platform === 'win32') {
        // --- WINDOWS ---
        srcInExe = path.join(__dirname, 'dist_app', 'win-unpacked');
        destSystem = path.join(process.env.ProgramFiles || 'C:\\Program Files', 'Strongholder');
        exePath = path.join(destSystem, 'Strongholder.exe');
        const lnkPath = path.join(os.homedir(), 'Desktop', 'Strongholder.lnk');

        // Vérif
        if (!fs.existsSync(srcInExe)) throw new Error(`Dossier win-unpacked introuvable dans l'exe ! Avez-vous fait npm run build-app ?`);

        // Nettoyage et Copie
        if (fs.existsSync(destSystem)) try { fs.rmSync(destSystem, { recursive: true, force: true }); } catch (e) { }

        console.log(`   Copie vers ${destSystem}`);
        copyFolderSync(srcInExe, destSystem);

        // Raccourci Bureau
        const ps = `$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('${lnkPath}'); $s.TargetPath = '${exePath}'; $s.Save()`;
        await runCommand('powershell.exe', ['-Command', ps], 'Création Raccourci');

    } else {
        // --- LINUX ---
        srcInExe = path.join(__dirname, 'dist_app', 'linux-unpacked');
        destSystem = '/opt/strongholder';
        exePath = path.join(destSystem, 'strongholder'); // Vérifie si c'est majuscule ou minuscule selon ton build

        if (!fs.existsSync(srcInExe)) throw new Error(`Dossier linux-unpacked introuvable !`);
        if (fs.existsSync(destSystem)) fs.rmSync(destSystem, { recursive: true, force: true });

        copyFolderSync(srcInExe, destSystem);
        fs.chmodSync(exePath, 0o755); // Rendre exécutable

        // Raccourci
        const desktopFile = path.join(os.homedir(), 'Desktop', 'Strongholder.desktop');
        fs.writeFileSync(desktopFile, `[Desktop Entry]\nName=Strongholder\nExec=${exePath}\nType=Application\nIcon=utilities-terminal\n`);
        fs.chmodSync(desktopFile, 0o755);
    }

    return exePath;
}

// --- ETAPE 2 : CONFIGURATION SYSTEME & SCRIPTS ---
async function configureSystem() {
    console.log("\n⚙️  2. Configuration Système & Copie des scripts...");

    // 1. Extraction temporaire des scripts sur le disque
    if (!fs.existsSync(TEMP_DIR)) fs.mkdirSync(TEMP_DIR, { recursive: true });

    SCRIPT_FILES.forEach(f => {
        try {
            const src = path.join(__dirname, 'scripts', f);
            fs.writeFileSync(path.join(TEMP_DIR, f), fs.readFileSync(src));
        } catch (e) { console.warn(`   ⚠️ Script manquant dans l'installer: ${f}`); }
    });

    const platform = os.platform();

    if (platform === 'win32') {
        // === WINDOWS (WSL) ===

        // A. Préparation WSL (PowerShell)
        const prep = path.join(TEMP_DIR, 'prep_client.ps1');
        // await runCommand('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', prep], 'Prep Windows');

        // B. Copie des scripts vers /usr/local/sbin DANS WSL
        console.log("   Transfert des scripts vers WSL /usr/local/sbin...");

        for (const file of SCRIPT_FILES) {
            // On ne copie PAS les scripts d'installation, seulement les outils
            if (file.startsWith('prep_') || file.startsWith('install_')) continue;

            const sourceWsl = toWslPath(path.join(TEMP_DIR, file));

            // 1. Copie avec sudo
            await runCommand('wsl', ['sudo', 'cp', sourceWsl, `/usr/local/sbin/${file}`], `Copie ${file}`);
            // 2. Permissions +x
            await runCommand('wsl', ['sudo', 'chmod', '+x', `/usr/local/sbin/${file}`], `Chmod ${file}`);
            // 3. Conversion fin de ligne (optionnel mais recommandé si encodage foireux)
            // await runCommand('wsl', ['sudo', 'dos2unix', `/usr/local/sbin/${file}`], `Dos2Unix ${file}`);
        }

        // C. Lancement install finale
        // const installScript = toWslPath(path.join(TEMP_DIR, 'install_all_clients.sh'));
        await runCommand('wsl', ['sudo', 'bash', installScript], 'Install Finale');

    } else {
        // === LINUX ===

        // A. Préparation
        const prep = path.join(TEMP_DIR, 'prep_client.sh');
        await runCommand('sudo', ['bash', prep], 'Prep Linux');

        // B. Copie vers /usr/local/sbin
        console.log("   Copie des scripts vers /usr/local/sbin...");

        for (const file of SCRIPT_FILES) {
            if (file.startsWith('prep_') || file.startsWith('install_')) continue;

            const source = path.join(TEMP_DIR, file);
            await runCommand('sudo', ['cp', source, `/usr/local/sbin/${file}`], `Copie ${file}`);
            await runCommand('sudo', ['chmod', '+x', `/usr/local/sbin/${file}`], `Chmod ${file}`);
        }

        // C. Install finale
        const installScript = path.join(TEMP_DIR, 'install_all_clients.sh');
        await runCommand('sudo', ['bash', installScript], 'Install Finale');
    }

    // Nettoyage temp
    try { fs.rmSync(TEMP_DIR, { recursive: true, force: true }); } catch (e) { }
}

// --- MAIN ---
(async () => {
    try {
        console.log(`🚀 SETUP STRONGHOLDER (${os.platform()})`);

        // 1. Install App
        const appPath = await installApp();

        // 2. Install Scripts & WSL
        await configureSystem();

        console.log("\n✅ INSTALLATION RÉUSSIE !");
        console.log("🚀 Lancement de l'application...");

        const child = spawn(appPath, [], { detached: true, stdio: 'ignore' });
        child.unref();

        // On laisse 2 secondes pour voir le message de succès puis on ferme
        setTimeout(() => process.exit(0), 2000);

    } catch (e) {
        console.error("\n❌ ERREUR FATALE:", e.message);
        if (e.message.includes('EPERM')) console.error("⚠️  ASTUCE: Relancez l'installateur en tant qu'ADMINISTRATEUR.");

        console.log("Appuyez sur Entrée pour quitter...");
        process.stdin.resume();
        process.stdin.on('data', () => process.exit(1));
    }
})();