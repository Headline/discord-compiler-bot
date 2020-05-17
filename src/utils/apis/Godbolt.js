import fetch from 'node-fetch';
import CompilationService from './CompilationService'
import { Collection } from 'discord.js'

/**
 * @typedef {(string|GodboltCompiler)} CompilerResolvable
 */
/**
 * @typedef {(string|GodboltLanguage)} LanguageResolvable
 */
/**
 * A class designed to fetch & hold the list of valid
 * compilers from godbolt.
 * 
 * @extends {CompilationService}
 */
export class Godbolt extends CompilationService {
    /**
     * Creates a Compilers object.
     *
     * @param {CompilerClient} client compiler client
     */
    constructor(client) {
        super(client);
    }

    /**
     * Gets an item from the container
     * @param {string} id language id
     * @return {GodboltLanguage}
     */
    get(id) {
        return super.get(id);
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

            let langs = await response.json();
            for (const lang of langs) {
                let tmp = new GodboltLanguage(lang);
                this.set(tmp.id, tmp);
            }

            response = await fetch("https://godbolt.org/api/compilers/", {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
            });

            let compilers = await response.json();
            for (const compiler of compilers) {
                let tmp = new GodboltCompiler(compiler);
                let lang = this.get(tmp.lang);
                lang.set(tmp.id, tmp);
            }

            // dont emit under testing conditions
            if (this.client)
                this.client.emit('godboltReady');

        }
        catch (error) {
            throw (error); // throw it up
        }
    }

    /**
     * Returns the compiler from the given id
     * @param {CompilerResolvable} compiler 
     * @return {GodboltCompiler}
     */
    getCompiler(compiler) {
        if (compiler.id) {
            compiler = compiler.id;
        }
        return this.find((l) => l.has(compiler)).get(compiler);
    }

    /**
     * Returns if the compiler is valid
     * @param {CompilerResolvable} compiler 
     * @return {boolean}
     */
    isValidCompiler(compiler) {
        if (compiler.id) {
            compiler = compiler.id;
        }

        return this.find((l) => l.has(compiler)) != null;
    }

    /**
     * Determines if the language is valid, or if it resolves to a valid language
     * 
     * @param {LanguageResolvable} lang 
     */
    isValidLanguage(lang) {
        if (lang.id) {
            lang = lang.id;
        }

        return this.findLanguageByAlias(lang) != null;;
    }

    /**
     * Searches languages looking for a match based on alias
     * @param {string} alias
     * @return {GodboltLanguage}
     */
    findLanguageByAlias(alias) {
        alias = alias.toLowerCase();
        return this.find((l) => l.monaco == alias || l.name.toLowerCase() == alias || l.alias.includes(alias));
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
     * @param {Godbolt} godbolt
     * @param {String} code
     * @param {CompilerResolvable} compiler
     * @param {String} stdin
     * @param {Boolean} save
     * @param {string} compiler_option_raw
     * @param {Compilers} compilers
     */
    constructor(godbolt, code, compiler, compiler_option_raw) {
        if (compiler.id)
            compiler = compiler.id;
        // ensure compiler entry is valid
        if (!godbolt.isValidCompiler(compiler)) {
            // lets try to find out they inserted a language like c++, and then we can assume one...
            const lang = godbolt.findLanguageByAlias(compiler);
            if (!lang)
                throw new Error('Invalid language or compiler');
            compiler = lang.defaultCompiler;
        }

        this.compiler = godbolt.getCompiler(compiler);
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

    /**
     * Dispatches a request for godbolt using the given compiler
     * 
     * @throws {Error} throws if godbolt doesn't respond with 200
     * @returns {Promise<string[]>} first element of this array are the errors, if any. Second is the assembly
     */
    async dispatch() {
        try {
            const response = await fetch(this.compiler.getCompilationUrl(), {
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

/**
 * A Godbolt compiler
 */
export class GodboltCompiler {
    constructor(obj) {
        /**
         * Compiler id used for requests
         * @type {string}
         */
        this.id = '';
        /**
         * Compiler name
         * @type {string}
         */
        this.name = '';
        /**
         * Compiler aliases
         * @type {string[]}
         */
        this.alias = [];
        /**
         * Language id
         */
        this.lang = '';
        Object.assign(this, obj);
    }

    /**
     * Retrieves the object's api url for compilation executions
     */
    getCompilationUrl() {
        return `https://godbolt.org/api/compiler/${this}/compile`;
    }

    /**
     * Converts a GodboltCompiler to it's string representation, in all cases it's id
     */
    toString() {
        return this.id;
    }
}

/**
 * A Godbolt language which stores all of it's compilers
 * @extends {Collection}
 */
export class GodboltLanguage extends Collection {
    constructor(obj) {
        super();
        /**
         * Language display name
         * @type {string}
         */
        this.name = '';
        /**
         * unknown
         * @type {string}
         */
        this.monaco = '';
        /**
         * File extensions
         * @type {string[]}
         */
        this.extensions = [];
        /**
         * Language aliases
         * @type {string[]}
         */
        this.alias = [];
        /**
         * Godbolt id used for requests formatting
         * @type {string}
         */
        this.id = '';
        /**
         * Example string used for default display on Godbolt site
         * @type {string}
         */
        this.example = '';
        /**
         * Default compiler for the given language 
         * @type {string}
         */
        this.defaultCompiler = '';
        Object.assign(this, obj);
    }

    /**
     * Returns the default compiler for this language
     * @return {GodboltCompiler}
     */
    getDefaultCompiler() {
        return this.get(this.defaultCompiler);
    }

    /**
     * Retrieves a compiler from the language's collection
     * @param {CompilerResolvable} id 
     * @return {GodboltCompiler}
     */
    get(compiler) {
        if (compiler.id) {
            compiler = compiler.id;
        }

        return super.get(compiler);
    }
}