import { sptf } from './protos';
import { handleErrorCode } from './error_handling';

/** This method may throw */
async function login(username: string, password: string): Promise<string> {
    return Promise.resolve("");
}

/** This method does not throw */
async function loginWithCookie(): Promise<boolean> {
    return Promise.resolve(true);
}

/** This method may throw */
async function signup(username: string, password: string) {
    return Promise.resolve();
}

const WEBSOCKET_URL = "wss://evian-workstation.local";

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

async function createWebsocket(): Promise<WebSocket> {
    let websocket = new WebSocket(WEBSOCKET_URL);
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

export { login, loginWithCookie, signup, createWebsocket, handleWebsocketData, requestChangeDir };
