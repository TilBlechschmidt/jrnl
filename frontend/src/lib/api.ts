import { goto } from '$app/navigation';
import { browser } from '$app/environment';

const { MODE } = import.meta.env;
const LOCAL_URL = "http://127.0.0.1:8080";
const PROD_URL = browser ? window.location.origin : 'definitely-not-working';

export const ENDPOINT = MODE === 'development' ? LOCAL_URL : PROD_URL;

// TODO Check if bad status codes actually throw an error!

export interface Document {
    identifier: number,
    contents: string
}

export async function list(): Promise<Document[]> {
    const response = await fetch(`${ENDPOINT}/api/document`);
    return await response.json();
}

export async function read(identifier: number): Promise<string> {
    const response = await fetch(`${ENDPOINT}/api/document/${identifier}`);
    return await response.text();
}

export async function write(identifier: number, contents: string) {
    await fetch(`${ENDPOINT}/api/document/${identifier}`, {
        method: 'PUT',
        body: contents
    });
}

export function login() {
    console.log({ MODE, LOCAL_URL, PROD_URL, ENDPOINT });
    goto(`${ENDPOINT}/auth/login`);
}
