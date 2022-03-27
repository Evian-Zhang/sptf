declare global {
    interface Window {
        sptfAPI: {
            getCookie: () => Promise<string | null>,
            setCookie: (authToken: string) => Promise<void>,
            removeCookie: () => Promise<void>,
            login: (username: string, password: string) => Promise<string>,
            loginWithCookie: () => Promise<boolean>,
            logout: () => Promise<void>,
            signup: (username: string, password: string) => Promise<string>,
            uploadFiles: (currentDir: string, files: {fileName: string, path: string}[]) => Promise<void>,
            makeDirectory: (directoryPath: string) => Promise<string>,
            downloadFiles: (url: string) => Promise<void>,
        }
    }
}

export {}
