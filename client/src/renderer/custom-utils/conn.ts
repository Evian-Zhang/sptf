import { sptf } from './protos';
import { handleErrorCode, handleNonOkHttpResponse } from './error_handling';

const SERVER_DOMAIN = "https://evian-workstation.local:8766";

/** This method may throw */
async function login(username: string, password: string): Promise<string> {
    let rawResponse;
    try {
        rawResponse = await fetch(`${SERVER_DOMAIN}/login`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({username: username, password: password}),
            mode: "no-cors"
        });
    } catch {
        throw "连接失败";
    }
    if (!rawResponse.ok) {
        await handleNonOkHttpResponse(rawResponse);
    }
    const response = await rawResponse.json();
    return Promise.resolve(response.authToken);
}

async function logout() {
    let rawResponse;
    try {
        rawResponse = await fetch(`${SERVER_DOMAIN}/logout`, {
            method: "POST",
            mode: "no-cors",
            credentials: "include",
        });
    } catch {
        throw "连接失败";
    }
    if (!rawResponse.ok) {
        await handleNonOkHttpResponse(rawResponse);
    }
    return Promise.resolve();
}

/** This method does not throw */
async function loginWithCookie(): Promise<boolean> {
    let rawResponse;
    try {
        rawResponse = await fetch(`${SERVER_DOMAIN}/login_with_cookie`, {
            method: "POST",
            mode: "no-cors",
            credentials: "include",
        });
    } catch {
        throw "连接失败";
    }
    if (!rawResponse.ok) {
        await handleNonOkHttpResponse(rawResponse);
    }
    return Promise.resolve(true);
}

/** This method may throw */
async function signup(username: string, password: string) {
    let rawResponse;
    try {
        rawResponse = await fetch(`${SERVER_DOMAIN}/signup`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({username: username, password: password}),
            mode: "no-cors"
        });
    } catch {
        throw "连接失败";
    }
    if (!rawResponse.ok) {
        await handleNonOkHttpResponse(rawResponse);
    }
    return Promise.resolve();
}

// see https://dev.to/ndrbrt/wait-for-the-websocket-connection-to-be-open-before-sending-a-message-1h12
const waitForOpenConnection = (socket: WebSocket) => {
    return new Promise<void>((resolve, reject) => {
        const maxNumberOfAttempts = 10;
        const intervalTime = 200; //ms

        let currentAttempt = 0;
        const interval = setInterval(() => {
            if (currentAttempt > maxNumberOfAttempts - 1) {
                clearInterval(interval);
                reject('连接失败');
            } else if (socket.readyState === socket.OPEN) {
                clearInterval(interval);
                resolve();
            }
            currentAttempt++
        }, intervalTime);
    });
}

const WEBSOCKET_URL = "wss://evian-workstation.local:8766";

async function createWebsocket(authToken: string): Promise<WebSocket> {
    let websocket = new WebSocket(`${WEBSOCKET_URL}/ws?authToken=${authToken}`);
    await waitForOpenConnection(websocket);
    return websocket;
}

// @ts-ignore
async function handleWebsocketData(data: Blob): Promise<sptf.IListDirectoryResponse> {
    const dataArray = await new Response(data).arrayBuffer();
    let basicOutcomingResponse;
    try {
        basicOutcomingResponse = sptf.BasicOutcomingMessage.decode(new Uint8Array(dataArray));
    } catch {
        throw "数据格式错误";
    }
    if (basicOutcomingResponse.ListDirectoryResponse) {
        return Promise.resolve(basicOutcomingResponse.ListDirectoryResponse);
    } else if (basicOutcomingResponse.GeneralError) {
        const generalError = basicOutcomingResponse.GeneralError;
        throw handleErrorCode(generalError.errorCode);
    }
}

function requestChangeDir(websocket: WebSocket, target_dir_path: string) {
    const data = sptf.BasicIncomingMessage.encode({
        version: 1,
        ListDirectoryMessage: {
            path: target_dir_path
        }
    }).finish();

    websocket.send(data);
}

function downloadFiles(authToken: string, filePaths: string[]) {
    const anchor = document.createElement('a');
    anchor.href = `${SERVER_DOMAIN}/download?paths=${filePaths.join(',')}`;
    anchor.download = '';
    document.body.appendChild(anchor);
    anchor.click();
    document.body.removeChild(anchor);
}

async function uploadFiles(currentDir: string, files: {fileName: string, content: Blob}[]) {
    let uploadedFiles = [];
    for (let file of files) {
        const dataArray = await new Response(file.content).arrayBuffer();
        const content = new Uint8Array(dataArray)
        uploadedFiles.push({fileName: file.fileName, content: content});
    }
    const fileUploadRequest = sptf.FileUploadRequest.encode({
        dirPath: currentDir,
        uploadedFile: uploadedFiles
    });
    let rawResponse;
    try {
        rawResponse = await fetch(`${SERVER_DOMAIN}/upload`, {
            method: "POST",
            body: fileUploadRequest.finish(),
            mode: "no-cors",
            credentials: "include",
        });
    } catch {
        throw "连接失败";
    }
    if (!rawResponse.ok) {
        await handleNonOkHttpResponse(rawResponse);
    }
    return Promise.resolve();
}

async function makeDirectory(directoryPath: string) {
    let rawResponse;
    try {
        rawResponse = await fetch(`${SERVER_DOMAIN}/make_directory`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({directoryPath: directoryPath}),
            mode: "no-cors",
            credentials: "include",
        });
    } catch {
        throw "连接失败";
    }
    if (!rawResponse.ok) {
        await handleNonOkHttpResponse(rawResponse);
    }
    return Promise.resolve();
}

export { login, loginWithCookie, logout, signup, createWebsocket, handleWebsocketData, requestChangeDir, downloadFiles, uploadFiles, makeDirectory };
