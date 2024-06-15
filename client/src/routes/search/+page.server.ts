import { type Load, redirect } from '@sveltejs/kit';

const limit = 10;

export const load: Load = async ({ url, fetch }) => {
	const api = "http://localhost:8080/search";
	const query = url.searchParams.get("q");
	let page = url.searchParams.get("p");

	if (query == null) {
		throw redirect(301,"/");
	}

	if (page ==null) {
		page = "0"
	}
	const page_num = Number(page);
	const offset = limit * page_num;

	const res =  await fetch(api, {
		method: "POST",
		headers: new Headers({
			"Content-Type": "application/json"
		}),
		body: JSON.stringify({
			"query": query,
			"limit": limit,
			"offset": offset
		})
	});


	const json = await res.json();

	return {
		data: json
	}

};