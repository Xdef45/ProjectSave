const { app, BrowserWindow, ipcMain } = require("electron");
const express = require("express");

let lastKey = null;

function startLocalApi() {
  const api = express();
  api.use(express.json({ limit: "64kb" }));

  api.post("/api/tunnel-key", (req, res) => {
    const { clientId, publicKey } = req.body || {};
    if (!clientId || !publicKey) return res.status(400).send("Missing fields");
    if (!publicKey.startsWith("ssh-ed25519 ")) return res.status(400).send("Invalid key format");

    lastKey = { clientId, publicKey, receivedAt: new Date().toISOString() };
    console.log("✅ RECU:", lastKey);

    BrowserWindow.getAllWindows().forEach(win => {
      win.webContents.send("key-received", lastKey);
    });

    return res.sendStatus(204);
  });

  api.listen(3000, "127.0.0.1", () => console.log("API up: http://127.0.0.1:3000"));
}

function createWindow() {
  const win = new BrowserWindow({
    width: 900,
    height: 600,
    webPreferences: {
      preload: __dirname + "/preload.js",
      contextIsolation: true,
    },
  });
  win.loadFile("index.html");
}

app.whenReady().then(() => {
  startLocalApi();
  createWindow();
});

ipcMain.handle("get-last-key", () => lastKey);
