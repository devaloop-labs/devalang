import { expect } from "chai";

const core: any = require("../../out-tsc");

describe("playhead callback", () => {
  it("should call registered callback during run", () => {
    const register = core.pkg && core.pkg.register_playhead_callback;
    const unregister = core.pkg && core.pkg.unregister_playhead_callback;
    if (typeof register !== "function") {
      // skip when wasm not available
      return;
    }

    let called = false;
    const cb = (ev: any) => {
      called = true;
      expect(ev).to.have.property("time");
      expect(ev).to.have.property("line");
      expect(ev).to.have.property("column");
    };

    register(cb);

    // run a simple program to trigger events
    const program = `bpm 120\n\n.myTrigger 1/4\n`;
    // render_audio will schedule triggers and should invoke the callback during execution
    const render = core.pkg && core.pkg.render_audio;
    if (typeof render === "function") {
      render(program);
    }

    unregister && unregister();

    expect(called).to.be.true;
  });
});
