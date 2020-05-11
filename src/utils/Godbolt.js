import fetch from 'node-fetch';
import { Collection } from 'discord.js'

/**
 * A class designed to fetch & hold the list of valid
 * compilers from godbolt.
 */
export class Godbolt extends Collection {
    /**
     * Creates a Compilers object.
     *
     * @param {CompilerClient} client compiler client for events
     */
    constructor(client) {
        super();

        this.client = client;
    }

    /**
     * Asyncronously fetches the list of valid languages and populates our cache.
     * Note: This can throw
     */
    async initialize() {
        try {
            let response = await fetch("https://godbolt.org/api/languages", {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
            });

            this.langs = await response.json();

            response = await fetch("https://godbolt.org/api/compilers/", {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
            });

            this.compilers = await response.json();

            // dont emit under testing conditions
            if (this.client)
                this.client.emit('godboltReady');

        }
        catch (error) {
            throw (error); // throw it up
        }
    }

    isValidCompiler(compiler) {
        for (const comp of this.compilers) {
            if (comp.id == compiler)
                return true;
        }
    }

    findLanguageByAlias(alias) {
        alias = alias.toLowerCase();
        for (const lang of this.langs) {
            if (lang.monaco == alias || lang.name.toLowerCase() == alias || lang.alias.includes(alias)) {
                return lang;
            }
        }
        return null;
    }

    getDefaultCompiler(input) {
        let lang = this.findLanguageByAlias(input);
        return (lang)?lang.defaultCompiler:null;
    }
}

/**
 * Class which represents all the settings and information for a single compilation
 * request. This should be built and used in coordination with Compiler.
 */
export class GodboltSetup {
    /**
     * Creates a compilation setup for usage with the Compiler object.
     * You may pass a language instead of a compiler for the second parameter,
     * and it will be compiled with the first compiler found in the list. The compiler
     * used is #1 on the menu for ;compilers <lang>.
     * @param {GodboltLangs} godboltlangs
     * @param {String} code
     * @param {String} compiler
     * @param {String} stdin
     * @param {Boolean} save
     * @param {string} compiler_option_raw
     * @param {Compilers} compilers
     */
    constructor(godboltlangs, code, compiler, compiler_option_raw) {
        // ensure compiler entry is valid
        if (!godboltlangs.isValidCompiler(compiler)) {
            // lets try to find out they inserted a language like c++, and then we can assume one...
            compiler = godboltlangs.getDefaultCompiler(compiler)
            if (!compiler)
                throw new Error('Invalid language or compiler');
        }

        this.compiler = compiler;
        this.source = code;
        this.options = {
            userArguments: compiler_option_raw,
            compilerOptions: {},
            filters: {
                binary: false,
                commentOnly: true,
                demangle: true,
                directives: true,
                execute: false,
                intel: true,
                labels: true,
                libraryCode: false,
                trim: false
            }
        }
    }

    async dispatch() {
        try {
            const response = await fetch("https://godbolt.org/api/compiler/"+this.compiler+'/compile', {
                method: "POST",
                body: JSON.stringify(this),
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
            });

            // We have a request error so lets throw up to our handler
            // which prints the output in an embed
            if (!response.ok)
                throw new Error(`Godbolt replied with response code ${response.status}. `
                + `This could mean Godbolt is experiencing an outage, or a network connection error has occured`);

            const json = await response.json();

            let errors = [];
            json.stderr.forEach((obj) => {
                errors.push(obj.text);
            })
            let asm = [];
            json.asm.forEach((obj) => {
                asm.push(obj.text);
            })

            const joinedErrors = errors.join('\n');
            return [(joinedErrors.trim().length == 0)?null:joinedErrors, asm.join('\n')];
        }
        catch (error) {
            throw(error); // rethrow to higher level
        }
    }
}