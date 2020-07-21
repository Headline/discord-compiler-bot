import { Collection } from 'discord.js'


/**
 * Helper class to handle regex replacement
 */
export class FixerEntry {
    /**
     * Creates a fixer entry
     * @param {RegExp} regex 
     * @param {string} replace 
     */
    constructor(regex, replace) {
        this.regex = regex;
        this.replace = replace;
    }

    /**
     * Performs a regex replacement, fixing the input string.
     * @param {string} string replaced str
     */
    fix(string) {
        return string.replace(this.regex, this.replace);
    }
};

/**
 * Helper class to try and catch common compilation errors unique to this environment
 * 
 * @extends {Collection}
 */
export class CompilationFixer extends Collection {
    constructor(client) {
        super();

        let fixer = new FixerEntry(/public (class)/g, '$1');
        this.set('java', [fixer]);
    }
}