import "regenerator-runtime/runtime.js";

import MessageTimeout from '../utils/MessageTimeout'

class testobj {
    constructor(done) {
        this.done = done;
        this.reactions = {
            cache: {
                forEach(funct) {
                    // fake stub for loop
                }
            }
        }
    }
    stop() {
        this.done();
    };
}
describe('MessageTimeout', function () {
    this.timeout(6000)
    it('1 sec timer', function (done)  {
        let obj = new testobj(done);
        let timeout = new MessageTimeout(obj, obj, 1);
        timeout.start()
    });
    it('5 sec timer restart', function (done)  {
        let obj = new testobj(done);
        let timeout = new MessageTimeout(obj, obj, 5);
        timeout.start()
        timeout.restart();
    });
});