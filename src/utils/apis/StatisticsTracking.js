import fetch from 'node-fetch'
import CompilerClient from '../../CompilerClient'
import log from '../../log'

/**
 * Internal class to handle statistics api requests
 */
export class StatisticsAPI {
    /**
     * Creates an object which represents the server count tracking
     * 
     * @param {CompilerClient} client
     * @param {string} url
     */
    constructor(client, url) {
        /**
         * API Key for request authentication
         * 
         * @type {string}
         */
        this.key = process.env.STATS_API_KEY;

        /**
         * Discord client
         * 
         * @type {CompilerClient}
         */        
        this.client = client;

        /**
         * Stats API url
         * 
         * @type {string}
         */
        this.url = url;
    }

    /**
     * Informs the API that a command has been used
     * 
     * @param {string} cmd command which has been executed
     */
    async commandExecuted(cmd) {
        try {
            let obj = {
                key: this.key,
                command: cmd
            };

            const response = await fetch(this.url + 'insert/command', {
                method: "POST",
                body: JSON.stringify(obj),
                headers: {
                    'Content-Type': 'application/json; charset=utf-8'
                },
            });

            if (!response.ok) {
                let resp = await response.json();
                log.warn(`StatisticsAPI#commandExecuted (response) -> ${resp.message}`);
            }
        }
        catch (error) {
            log.error(`StatisticsAPI#commandExecuted -> ${error.message}`);
        }
    }

    /**
     * Informs the API which language has just been compiled
     * 
     * @param {string} lang langauge compiled
     * @param {boolean} failure indicates whether it was a failed compilation
     */
    async compilationExecuted(lang, failure) {
        // if we were given a compiler we need to find the langauge
        if (!this.client.wandbox.has(lang)) {
            this.client.wandbox.forEach((value, key, map) => {
                if (value.includes(lang)) {
                    lang = key;
                }
            });
        }

        try {
            let obj = {
                key: this.key,
                language: lang,
                fail: failure
            };

            const response = await fetch(this.url + 'insert/language', {
                method: "POST",
                body: JSON.stringify(obj),
                headers: {
                    'Content-Type': 'application/json; charset=utf-8'
                },
            });

            if (!response.ok) {
                let resp = await response.json();
                log.warn(`StatisticsAPI#compilationExecuted (response) -> ${resp.message}`);
            }
        }
        catch (error) {
            log.error(`StatisticsAPI#compilationExecuted -> ${error.message}`);
        }
    }

    /**
     * Increments the request count information by one
     */
    async incrementRequestCount() {
        try {
            let obj = {
                key: this.key,
                type: 'request'
            };

            const response = await fetch(this.url + 'insert/legacy', {
                method: "POST",
                body: JSON.stringify(obj),
                headers: {
                    'Content-Type': 'application/json; charset=utf-8'
                },
            });

            if (!response.ok) {
                let resp = await response.json();
                log.warn(`StatisticsAPI#incrementRequestCount (response) -> ${resp.message}`);
            }
        }
        catch (error) {
            log.error(`StatisticsAPI#incrementRequestCount -> ${error.message}`);
        }
    }

    /**
     * Feeds API server count information
     * @param {number} count 
     */
    async insertServerCount(count) {
        try {
            let obj = {
                key: this.key,
                amount: count,
                type: 'server'
            };

            const response = await fetch(this.url + 'insert/legacy', {
                method: "POST",
                body: JSON.stringify(obj),
                headers: {
                    'Content-Type': 'application/json; charset=utf-8'
                },
            });

            if (!response.ok) {
                let resp = await response.json();
                log.warn(`StatisticsAPI#insertServerCount (response) -> ${resp.message}`);
            }
        }
        catch (error) {
            log.error(`StatisticsAPI#insertServerCount -> ${error.message}`);
        }
    }
}