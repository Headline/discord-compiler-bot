import CompilerClient from '../CompilerClient'


/** When discord.js does their ts rewrite - this should probably go away */

/**
 * Internal timer helper that expires old help command outputs
 */
export default class MemorySweeper {
    /**
     * Creates a MemorySweeper object which periodically sweeps d.js caches
     * 
     * @param {CompilerClient} message message to remove reactions from after delay
     * @param {number} delay delay in minutes to sweep memory
     */
    constructor(client, delay) {
        this.delay = delay * 60;
        this.client = client;
        this.start();
    }

    /**
     * Starts the message timeout timer
     */
    start() {
        this.timeout = setInterval(this.run, this.delay * 1000, this.client);
    }

    /**
     * Clears discord.js caches
     * 
     * @param {CompilerClient} client 
     */
    run(client) {
        // 1hr ago
        const comparisonDate = Date.now() - 1000 * 60 * 60;

        let deleted = 0;
        deleted = client.users.cache.sweep((usr) => {
            return usr.createdAt > comparisonDate;
        });
        deleted += client.channels.cache.sweep((chan) => {
            return chan.createdAt > comparisonDate;
        });

      /**An error has occured in ${name} command
       * Event called when client sweeps it's own memory
       * 
       * @event CompilerClient#memorySweeped
       * @type {number} number of objects sweeped
       */
        client.emit('memorySweeped', deleted);
    }
}