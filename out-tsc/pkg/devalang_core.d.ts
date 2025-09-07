export declare function parse(...args: any[]): any;
export declare function debug_render(...args: any[]): any;
export declare function render_audio(...args: any[]): any;
export declare function register_playhead_callback(cb?: any): any;
export declare function collect_playhead_events(): any;
export declare function unregister_playhead_callback(): any;
declare const pkg: {
    parse: typeof parse;
    debug_render: typeof debug_render;
    render_audio: typeof render_audio;
    register_playhead_callback: typeof register_playhead_callback;
    collect_playhead_events: typeof collect_playhead_events;
    unregister_playhead_callback: typeof unregister_playhead_callback;
};
export default pkg;
