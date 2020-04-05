import fs from 'fs'
import { spawn } from 'child_process'

/**
 * Manages server count for when statistics tracking & status display.
 * Functions like |updateSite| have no effect if the graph.py doesn't exist,
 * which is only useful for the main public instance. These statistics help
 * track the bot's growth along with any other information that we deem to be
 * relevant.
 */
export class Servers {
    /**
     * Creates an object which represents the server count tracking
     * 
     * @param {Number} count 
     * @param {Client} client
     * @param {dbl} dbl 
     */
    constructor(count, client, dbl) {
        this.count = count;
        this.client = client;
        this.dbl = dbl;
    }

    /**
     * Sets the server count to a new value
     * 
     * Note: this does not contain a call to updateAll()
     * 
     * @param {Number} count 
     */
    setCount(count) {
        this.count = count;
    }

    /**
     * Gets the current server count
     */
    getCount() {
        return this.count;
    }

    /**
     * Updates discordbots.org server count with the supplied value. Only functions on
     * instance.
     * @param {Number} count 
     */
    updateDBL(count) {
        if (this.dbl)
            this.dbl.postStats(count);
    }

    /**
     * Updates the website stats with the supplied server count
     * @param {Number} count 
     */
    updateSite(count) {
        let file = '/var/www/html/discord-compiler/graph.py';
        fs.stat(file, (err) => {
            if (err == null) {
                spawn('python', [file, 'servers', String(count)]);
            }
        });
    }

    /**
     * Updates the discord presence with the supplied server count
     * @param {Number} count 
     */
    async updateDiscord(count) {
        await this.client.user.setPresence({ activity: { name: `in ${count} servers | ;help`}, status: 'online'})
    }

    /**
     * Updates both the website & the discord presence with latest count
     */
    async updateAll() {
        this.updateSite(this.count);
        await this.updateDiscord(this.count);
    }   

    /**
     * Increments the server count & updates all
     */
    async inc() {
        this.count++;
        await this.updateAll();
    }

    /**
     * Decrements the server count and updates all
     */
    async dec() {
        this.count--;
        await this.updateAll();
    }
}

/**
 * Simple singleton class which contains stats tracking to be done
 * on a request-by-request basis.
 */
export class Requests {
    /**
     * Increments the stats request count by one. Like before,
     * this has no effect if run outside of the public bot environment.
     */
    static doRequest() {
        let file = '/var/www/html/discord-compiler/graph.py';
        fs.stat(file, (err, stat) => {
            if (err == null) {
                spawn('python', [file]);
            }
        });
    }
}