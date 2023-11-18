import { read } from "$lib/api";

/** @type {import('./$types').PageLoad} */
export async function load({ params }) {
	const identifier = parseInt(params.documentID);
	const contents = await read(identifier);
	return { identifier, contents };
}