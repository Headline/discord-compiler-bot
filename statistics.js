const fs = require('fs');
const spawn = require('child_process').spawn;

/**
 * Manages server count for when statistics tracking & status display.
 * Functions like |updateSite| have no effect if the graph.py doesn't exist,
 * which is only useful for the main public instance. These statistics help
 * track the bot's growth along with any other information that we deem to be
 * relevant.
 */
class Servers {
    /**
     * Creates an object which represents the server count tracking
     * 
     * @param {Number} count 
     * @param {Discord.Client} client
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
    updateDiscord(count) {
        this.client.user.setPresence({ game: { name: `in ${count} servers | ;help`}, status: 'online'})
        .catch(console.log);
    }

    /**
     * Updates both the website & the discord presence with latest count
     */
    updateAll() {
        this.updateSite(this.count);
        this.updateDiscord(this.count);
    }   

    /**
     * Increments the server count & updates all
     */
    inc() {
        this.count++;
        this.updateAll();
    }

    /**
     * Decrements the server count and updates all
     */
    dec() {
        this.count--;
        this.updateAll();
    }
}

/**
 * Simple singleton class which contains stats tracking to be done
 * on a request-by-request basis.
 */
class Requests {
    /**
     * Increments the stats request count by one. Like before,
     * this has no effect if run outside of the public bot environment.
     */
    static DoRequest() {
        let file = '/var/www/html/discord-compiler/graph.py';
        fs.stat(file, (err, stat) => {
            if (err == null) {
                spawn('python', [file]);
            }
        });
    }
}

module.exports = {Servers, Requests};