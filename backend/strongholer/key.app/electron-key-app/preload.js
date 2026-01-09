const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("api", {
  getLastKey: () => ipcRenderer.invoke("get-last-key"),
  onKeyReceived: (cb) => ipcRenderer.on("key-received", (_evt, data) => cb(data)),
});
