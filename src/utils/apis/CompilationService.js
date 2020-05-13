import { Client, Collection } from 'discord.js'

/**
 * Abstract class that represents a compilation service
 * @extends {Collection}
 * @abstract
 */
export default class CompilationService extends Collection {
    constructor(client) {
        super();

        /**
         * @type {Client}
         */
        this.client = client;
    }

    /**
     * Abstract language validation to be implemented by base classes
     * 
     * @abstract
     * @param {string} lang language id
     */
    isValidLanguage(lang) {
        throw new Error(`${this.constructor.name} has no isValidLang method.`); 
    }

    /**
     * Abstract method to initialize a CompilationService's resources
     * 
     * @abstract
     */
    async initialize() {
        throw new Error(`${this.constructor.name} has no initialize method.`); 
    }
    
    /**
     * Abstract compiler validation to be implemented by base classes
     * 
     * @abstract
     * @param {string} compiler compiler id
     */
    isValidCompiler(compiler) {
        throw new Error(`${this.constructor.name} has no isValidCompiler method.`); 
    }
}