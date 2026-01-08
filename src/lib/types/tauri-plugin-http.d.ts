// Type declarations for @tauri-apps/plugin-http
declare module '@tauri-apps/plugin-http' {
    export interface FetchOptions {
        method?: string;
        headers?: Record<string, string>;
        body?: string | Uint8Array;
    }

    export interface FetchResponse {
        ok: boolean;
        status: number;
        statusText: string;
        headers: Record<string, string>;
        text(): Promise<string>;
        json(): Promise<unknown>;
        bytes(): Promise<Uint8Array>;
    }

    export function fetch(url: string, options?: FetchOptions): Promise<FetchResponse>;
}
