declare global {
    interface Window {
        sptfAPI: {
            getCookie: () => Promise<string | null>,
            setCookie: (authToken: string) => Promise<void>,
            removeCookie: () => Promise<void>
        }
    }
}

export {}
