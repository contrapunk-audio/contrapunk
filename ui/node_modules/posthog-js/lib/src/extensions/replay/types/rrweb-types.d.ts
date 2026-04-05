export type DataURLOptions = Partial<{
    type: string;
    quality: number;
}>;
export interface INode extends Node {
    __sn: serializedNodeWithId;
}
export interface IMirror<TNode> {
    getId(n: TNode | undefined | null): number;
    getNode(id: number): TNode | null;
    getIds(): number[];
    getMeta(n: TNode): serializedNodeWithId | null;
    removeNodeFromMap(n: TNode): void;
    has(id: number): boolean;
    hasNode(node: TNode): boolean;
    add(n: TNode, meta: serializedNodeWithId): void;
    replace(id: number, n: TNode): void;
    reset(): void;
}
export declare class Mirror implements IMirror<Node> {
    private idNodeMap;
    private nodeMetaMap;
    getId(n: Node | undefined | null): number;
    getNode(id: number): Node | null;
    getIds(): number[];
    getMeta(n: Node): serializedNodeWithId | null;
    removeNodeFromMap(n: Node): void;
    has(id: number): boolean;
    hasNode(node: Node): boolean;
    add(n: Node, meta: serializedNodeWithId): void;
    replace(id: number, n: Node): void;
    reset(): void;
}
export type attributes = {
    [key: string]: string | number | true | null;
};
export declare const NodeType: {
    readonly Document: 0;
    readonly DocumentType: 1;
    readonly Element: 2;
    readonly Text: 3;
    readonly CDATA: 4;
    readonly Comment: 5;
};
export type NodeType = (typeof NodeType)[keyof typeof NodeType];
export type documentNode = {
    type: typeof NodeType.Document;
    childNodes: serializedNodeWithId[];
    compatMode?: string;
};
export type documentTypeNode = {
    type: typeof NodeType.DocumentType;
    name: string;
    publicId: string;
    systemId: string;
};
export type elementNode = {
    type: typeof NodeType.Element;
    tagName: string;
    attributes: attributes;
    childNodes: serializedNodeWithId[];
    isSVG?: true;
    needBlock?: boolean;
    isCustom?: true;
};
export type textNode = {
    type: typeof NodeType.Text;
    textContent: string;
    isStyle?: true;
};
export type cdataNode = {
    type: typeof NodeType.CDATA;
    textContent: '';
};
export type commentNode = {
    type: typeof NodeType.Comment;
    textContent: string;
};
export type serializedNode = (documentNode | documentTypeNode | elementNode | textNode | cdataNode | commentNode) & {
    rootId?: number;
    isShadowHost?: boolean;
    isShadow?: boolean;
};
export type serializedNodeWithId = serializedNode & {
    id: number;
};
export type blockClass = string | RegExp;
export type maskTextClass = string | RegExp;
export type IWindow = Window & typeof globalThis;
export type listenerHandler = () => void;
export type KeepIframeSrcFn = (src: string) => boolean;
export type PackFn = (event: eventWithTime) => string;
export declare const EventType: {
    readonly DomContentLoaded: 0;
    readonly Load: 1;
    readonly FullSnapshot: 2;
    readonly IncrementalSnapshot: 3;
    readonly Meta: 4;
    readonly Custom: 5;
    readonly Plugin: 6;
};
export type EventType = (typeof EventType)[keyof typeof EventType];
export declare const IncrementalSource: {
    readonly Mutation: 0;
    readonly MouseMove: 1;
    readonly MouseInteraction: 2;
    readonly Scroll: 3;
    readonly ViewportResize: 4;
    readonly Input: 5;
    readonly TouchMove: 6;
    readonly MediaInteraction: 7;
    readonly StyleSheetRule: 8;
    readonly CanvasMutation: 9;
    readonly Font: 10;
    readonly Log: 11;
    readonly Drag: 12;
    readonly StyleDeclaration: 13;
    readonly Selection: 14;
    readonly AdoptedStyleSheet: 15;
    readonly CustomElement: 16;
};
export type IncrementalSource = (typeof IncrementalSource)[keyof typeof IncrementalSource];
export type domContentLoadedEvent = {
    type: typeof EventType.DomContentLoaded;
    data: unknown;
};
export type loadedEvent = {
    type: typeof EventType.Load;
    data: unknown;
};
export type fullSnapshotEvent = {
    type: typeof EventType.FullSnapshot;
    data: {
        node: serializedNodeWithId;
        initialOffset: {
            top: number;
            left: number;
        };
    };
};
export type metaEvent = {
    type: typeof EventType.Meta;
    data: {
        href: string;
        width: number;
        height: number;
    };
};
export type customEvent<T = unknown> = {
    type: typeof EventType.Custom;
    data: {
        tag: string;
        payload: T;
    };
};
export type pluginEvent<T = unknown> = {
    type: typeof EventType.Plugin;
    data: {
        plugin: string;
        payload: T;
    };
};
export type styleOMValue = {
    [key: string]: styleValueWithPriority | string | false;
};
export type styleValueWithPriority = [string, string];
export type textMutation = {
    id: number;
    value: string | null;
};
export type attributeMutation = {
    id: number;
    attributes: {
        [key: string]: string | styleOMValue | null;
    };
};
export type removedNodeMutation = {
    parentId: number;
    id: number;
    isShadow?: boolean;
};
export type addedNodeMutation = {
    parentId: number;
    previousId?: number | null;
    nextId: number | null;
    node: serializedNodeWithId;
};
export type mutationCallbackParam = {
    texts: textMutation[];
    attributes: attributeMutation[];
    removes: removedNodeMutation[];
    adds: addedNodeMutation[];
    isAttachIframe?: true;
};
export type mutationData = {
    source: typeof IncrementalSource.Mutation;
} & mutationCallbackParam;
export type mousePosition = {
    x: number;
    y: number;
    id: number;
    timeOffset: number;
};
export declare const MouseInteractions: {
    readonly MouseUp: 0;
    readonly MouseDown: 1;
    readonly Click: 2;
    readonly ContextMenu: 3;
    readonly DblClick: 4;
    readonly Focus: 5;
    readonly Blur: 6;
    readonly TouchStart: 7;
    readonly TouchMove_Departed: 8;
    readonly TouchEnd: 9;
    readonly TouchCancel: 10;
};
export type MouseInteractions = (typeof MouseInteractions)[keyof typeof MouseInteractions];
export declare const PointerTypes: {
    readonly Mouse: 0;
    readonly Pen: 1;
    readonly Touch: 2;
};
export type PointerTypes = (typeof PointerTypes)[keyof typeof PointerTypes];
type mouseInteractionParam = {
    type: MouseInteractions;
    id: number;
    x?: number;
    y?: number;
    pointerType?: PointerTypes;
};
export type mouseInteractionData = {
    source: typeof IncrementalSource.MouseInteraction;
} & mouseInteractionParam;
export type mousemoveData = {
    source: typeof IncrementalSource.MouseMove | typeof IncrementalSource.TouchMove | typeof IncrementalSource.Drag;
    positions: mousePosition[];
};
export type scrollPosition = {
    id: number;
    x: number;
    y: number;
};
export type scrollData = {
    source: typeof IncrementalSource.Scroll;
} & scrollPosition;
export type viewportResizeDimension = {
    width: number;
    height: number;
};
export type viewportResizeData = {
    source: typeof IncrementalSource.ViewportResize;
} & viewportResizeDimension;
export type inputValue = {
    text: string;
    isChecked: boolean;
    userTriggered?: boolean;
};
export type inputData = {
    source: typeof IncrementalSource.Input;
    id: number;
} & inputValue;
export declare const MediaInteractions: {
    readonly Play: 0;
    readonly Pause: 1;
    readonly Seeked: 2;
    readonly VolumeChange: 3;
    readonly RateChange: 4;
};
export type MediaInteractions = (typeof MediaInteractions)[keyof typeof MediaInteractions];
export type mediaInteractionParam = {
    type: MediaInteractions;
    id: number;
    currentTime?: number;
    volume?: number;
    muted?: boolean;
    loop?: boolean;
    playbackRate?: number;
};
export type mediaInteractionData = {
    source: typeof IncrementalSource.MediaInteraction;
} & mediaInteractionParam;
export type styleSheetAddRule = {
    rule: string;
    index?: number | number[];
};
export type styleSheetDeleteRule = {
    index: number | number[];
};
export type styleSheetRuleParam = {
    id?: number;
    styleId?: number;
    removes?: styleSheetDeleteRule[];
    adds?: styleSheetAddRule[];
    replace?: string;
    replaceSync?: string;
};
export type styleSheetRuleData = {
    source: typeof IncrementalSource.StyleSheetRule;
} & styleSheetRuleParam;
export declare const CanvasContext: {
    readonly '2D': 0;
    readonly WebGL: 1;
    readonly WebGL2: 2;
};
export type CanvasContext = (typeof CanvasContext)[keyof typeof CanvasContext];
export type canvasMutationCommand = {
    property: string;
    args: Array<unknown>;
    setter?: true;
};
export type canvasMutationParam = {
    id: number;
    type: CanvasContext;
    commands: canvasMutationCommand[];
} | ({
    id: number;
    type: CanvasContext;
} & canvasMutationCommand);
export type canvasMutationData = {
    source: typeof IncrementalSource.CanvasMutation;
} & canvasMutationParam;
export type fontParam = {
    family: string;
    fontSource: string;
    buffer: boolean;
    descriptors?: FontFaceDescriptors;
};
export type fontData = {
    source: typeof IncrementalSource.Font;
} & fontParam;
export type SelectionRange = {
    start: number;
    startOffset: number;
    end: number;
    endOffset: number;
};
export type selectionParam = {
    ranges: Array<SelectionRange>;
};
export type selectionData = {
    source: typeof IncrementalSource.Selection;
} & selectionParam;
export type styleDeclarationParam = {
    id?: number;
    styleId?: number;
    index: number[];
    set?: {
        property: string;
        value: string | null;
        priority: string | undefined;
    };
    remove?: {
        property: string;
    };
};
export type styleDeclarationData = {
    source: typeof IncrementalSource.StyleDeclaration;
} & styleDeclarationParam;
export type adoptedStyleSheetParam = {
    id: number;
    styles?: {
        styleId: number;
        rules: styleSheetAddRule[];
    }[];
    styleIds: number[];
};
export type adoptedStyleSheetData = {
    source: typeof IncrementalSource.AdoptedStyleSheet;
} & adoptedStyleSheetParam;
export type customElementParam = {
    define?: {
        name: string;
    };
};
export type customElementData = {
    source: typeof IncrementalSource.CustomElement;
} & customElementParam;
export type incrementalData = mutationData | mousemoveData | mouseInteractionData | scrollData | viewportResizeData | inputData | mediaInteractionData | styleSheetRuleData | canvasMutationData | fontData | selectionData | styleDeclarationData | adoptedStyleSheetData | customElementData;
export type incrementalSnapshotEvent = {
    type: typeof EventType.IncrementalSnapshot;
    data: incrementalData;
};
export type eventWithoutTime = domContentLoadedEvent | loadedEvent | fullSnapshotEvent | incrementalSnapshotEvent | metaEvent | customEvent | pluginEvent;
export type eventWithTime = eventWithoutTime & {
    timestamp: number;
    delay?: number;
};
export type mutationCallBack = (m: mutationCallbackParam) => void;
export type mousemoveCallBack = (p: mousePosition[], source: typeof IncrementalSource.MouseMove | typeof IncrementalSource.TouchMove | typeof IncrementalSource.Drag) => void;
export type mouseInteractionCallBack = (d: mouseInteractionParam) => void;
export type scrollCallback = (p: scrollPosition) => void;
export type viewportResizeCallback = (d: viewportResizeDimension) => void;
export type inputCallback = (v: inputValue & {
    id: number;
}) => void;
export type mediaInteractionCallback = (p: mediaInteractionParam) => void;
export type styleSheetRuleCallback = (s: styleSheetRuleParam) => void;
export type styleDeclarationCallback = (s: styleDeclarationParam) => void;
export type canvasMutationCallback = (p: canvasMutationParam) => void;
export type fontCallback = (p: fontParam) => void;
export type selectionCallback = (p: selectionParam) => void;
export type customElementCallback = (c: customElementParam) => void;
export type adoptedStyleSheetCallback = (a: adoptedStyleSheetParam) => void;
export type hooksParam = {
    mutation?: mutationCallBack;
    mousemove?: mousemoveCallBack;
    mouseInteraction?: mouseInteractionCallBack;
    scroll?: scrollCallback;
    viewportResize?: viewportResizeCallback;
    input?: inputCallback;
    mediaInteaction?: mediaInteractionCallback;
    styleSheetRule?: styleSheetRuleCallback;
    styleDeclaration?: styleDeclarationCallback;
    canvasMutation?: canvasMutationCallback;
    font?: fontCallback;
    selection?: selectionCallback;
    customElement?: customElementCallback;
};
export type SamplingStrategy = Partial<{
    mousemove: boolean | number;
    mousemoveCallback: number;
    mouseInteraction: boolean | Record<string, boolean | undefined>;
    scroll: number;
    media: number;
    input: 'all' | 'last';
    canvas: 'all' | number;
}>;
export interface ICrossOriginIframeMirror {
    getId(iframe: HTMLIFrameElement, remoteId: number, parentToRemoteMap?: Map<number, number>, remoteToParentMap?: Map<number, number>): number;
    getIds(iframe: HTMLIFrameElement, remoteId: number[]): number[];
    getRemoteId(iframe: HTMLIFrameElement, parentId: number, map?: Map<number, number>): number;
    getRemoteIds(iframe: HTMLIFrameElement, parentId: number[]): number[];
    reset(iframe?: HTMLIFrameElement): void;
}
export type RecordPlugin<TOptions = unknown> = {
    name: string;
    observer?: (cb: (...args: Array<unknown>) => void, win: IWindow, options: TOptions) => listenerHandler;
    eventProcessor?: <TExtend>(event: eventWithTime) => eventWithTime & TExtend;
    getMirror?: (mirrors: {
        nodeMirror: Mirror;
        crossOriginIframeMirror: ICrossOriginIframeMirror;
        crossOriginIframeStyleMirror: ICrossOriginIframeMirror;
    }) => void;
    options: TOptions;
};
export {};
