import { list } from "$lib/api";

export async function load() {
	return { documents: await list() };
}