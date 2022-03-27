const { contextBridge, ipcRenderer } = require('electron');

// see https://github.com/electron/electron/issues/24427
const decodeError = ({name, message, extra}) => {
  return extra
}

const invokeWithCustomErrors = async (...args) => {
  const {error, result} = await ipcRenderer.invoke(...args)
  if (error) {
    throw decodeError(error)
  }
  return result
}

contextBridge.exposeInMainWorld('sptfAPI', {
  getCookie: () => invokeWithCustomErrors('sptf:getCookie'),
  setCookie: (authToken) => invokeWithCustomErrors('sptf:setCookie', authToken),
  removeCookie: () => invokeWithCustomErrors('sptf:removeCookie'),
  login: (username, password) => invokeWithCustomErrors('sptf:login', username, password),
  loginWithCookie: () => invokeWithCustomErrors('sptf:loginWithCookie'),
  logout: () => invokeWithCustomErrors('sptf:logout'),
  signup: (username, password) => invokeWithCustomErrors('sptf:signup', username, password),
  uploadFiles: (currentDir, files) => invokeWithCustomErrors('sptf:uploadFiles', currentDir, files),
  downloadFiles: (url) => ipcRenderer.send('sptf:downloadFiles', url),
  makeDirectory: (directoryPath) => invokeWithCustomErrors('sptf:makeDirectory', directoryPath)
});
