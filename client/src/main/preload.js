const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('sptfAPI', {
  getCookie: () => ipcRenderer.invoke('sptf:getCookie'),
  setCookie: (authToken) => ipcRenderer.invoke('sptf:setCookie')
});
