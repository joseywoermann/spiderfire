/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

declare module "url" {
	export type FormatOptions = {
		auth?: boolean,
		fragment?: boolean,
		search?: boolean,
	};

	export function domainToASCII(domain: string): string;
	export function domainToUnicode(domain: string): string;

	export class URL {
		constructor(url: string, base?: string);

		get href(): string;
		set href(href: string);

		get protocol(): string;
		set protocol(protocol: string);

		get host(): string | null;
		set host(host: string | null | undefined);

		get hostname(): string | null;
		set hostname(hostname: string | null | undefined);

		get port(): number | null;
		set port(port: number | null | undefined);

		get path(): string;
		set path(path: string);

		get username(): string;
		set username(username: string);

		get password(): string | null;
		set password(password: string | null | undefined);

		get search(): string | null;
		set search(string: string | null | undefined);

		get hash(): string | null;
		set hash(hash: string | null | undefined);

		get origin(): string;

		toString(): string;
		toJSON(): string;
		format(options?: FormatOptions): string;
	}

	namespace Url {
		export {
			FormatOptions,

			domainToASCII,
			domainToUnicode,

			URL,
		};
	}

	export default Url;
}
