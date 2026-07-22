#import <AppKit/AppKit.h>
#import <CoreFoundation/CoreFoundation.h>

@class SSOCodeWindow;
static SSOCodeWindow *currentWindow = nil;
static CFRunLoopRef appRunLoop = nil;

// [NSApp stop:] only sets a flag checked on the next event, so we post a
// dummy event to ensure the run loop wakes and exits promptly.
static void stopRunLoop(void) {
    [NSApp stop:nil];
    NSEvent *dummy = [NSEvent otherEventWithType:NSEventTypeApplicationDefined
                                        location:NSMakePoint(0, 0)
                                   modifierFlags:0
                                       timestamp:0
                                    windowNumber:0
                                         context:nil
                                         subtype:0
                                           data1:0
                                           data2:0];
    [NSApp postEvent:dummy atStart:YES];
}

@interface SSOCodeWindow : NSObject
- (void)showWithCode:(NSString *)code onCloseRequested:(void (*)(void))callback;
- (void)closeWindow;
@end

@implementation SSOCodeWindow {
    NSWindow *_window;
    id _monitor;
    id _closeObserver;
    void (*_on_close_requested)(void);
}

- (void)showWithCode:(NSString *)code onCloseRequested:(void (*)(void))callback {
    _on_close_requested = callback;
    CGFloat xPadding = 36;
    CGFloat yPadding = 24;
    CGFloat innerSpacing = 8;

    NSFont *titleFont = [NSFont systemFontOfSize:13 weight:NSFontWeightRegular];
    NSFont *codeFont = [NSFont monospacedSystemFontOfSize:42 weight:NSFontWeightMedium];

    NSDictionary *codeAttrs = @{
        NSFontAttributeName: codeFont,
        NSKernAttributeName: @(4.0)
    };

    _window = [[NSWindow alloc] initWithContentRect:NSZeroRect
                                           styleMask:NSWindowStyleMaskBorderless
                                             backing:NSBackingStoreBuffered
                                               defer:NO];
    _window.releasedWhenClosed = NO;
    _window.movableByWindowBackground = YES;
    _window.opaque = NO;
    _window.backgroundColor = [NSColor clearColor];
    _window.level = NSFloatingWindowLevel;

    NSView *panel = [[NSView alloc] init];
    panel.wantsLayer = YES;
    panel.layer.backgroundColor = [[NSColor colorWithCalibratedRed:0 green:0 blue:0 alpha:0.85] CGColor];
    panel.layer.cornerRadius = 16;
    panel.layer.borderWidth = 2;
    panel.layer.borderColor = [[NSColor colorWithCalibratedWhite:1.0 alpha:0.25] CGColor];
    panel.translatesAutoresizingMaskIntoConstraints = NO;

    NSTextField *titleLabel = [NSTextField labelWithString:@"AWS SSO code"];
    titleLabel.font = titleFont;
    titleLabel.alignment = NSTextAlignmentCenter;
    titleLabel.textColor = [NSColor colorWithCalibratedWhite:1.0 alpha:0.6];

    NSTextField *codeLabel = [NSTextField labelWithString:@""];
    codeLabel.attributedStringValue = [[NSAttributedString alloc] initWithString:code attributes:codeAttrs];
    codeLabel.alignment = NSTextAlignmentCenter;
    codeLabel.textColor = [NSColor whiteColor];

    NSStackView *stack = [NSStackView stackViewWithViews:@[titleLabel, codeLabel]];
    stack.orientation = NSUserInterfaceLayoutOrientationVertical;
    stack.spacing = innerSpacing;
    stack.translatesAutoresizingMaskIntoConstraints = NO;

    NSView *contentView = _window.contentView;
    [contentView addSubview:panel];
    [panel addSubview:stack];

    [NSLayoutConstraint activateConstraints:@[
        [panel.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
        [panel.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
        [panel.topAnchor constraintEqualToAnchor:contentView.topAnchor],
        [panel.bottomAnchor constraintEqualToAnchor:contentView.bottomAnchor],

        [stack.leadingAnchor constraintEqualToAnchor:panel.leadingAnchor constant:xPadding],
        [stack.trailingAnchor constraintEqualToAnchor:panel.trailingAnchor constant:-xPadding],
        [stack.topAnchor constraintEqualToAnchor:panel.topAnchor constant:yPadding],
        [stack.bottomAnchor constraintEqualToAnchor:panel.bottomAnchor constant:-yPadding],
    ]];

    NSSize fittingSize = contentView.fittingSize;
    [_window setContentSize:fittingSize];

    NSScreen *screen = [NSScreen mainScreen];
    if (!screen) {
        fprintf(stderr, "Error: no screen detected\n");
        exit(1);
    }
    NSRect screenFrame = screen.frame;
    [_window setFrameOrigin:NSMakePoint(
        NSMidX(screenFrame) - fittingSize.width / 2,
        NSMaxY(screenFrame) - screenFrame.size.height / 4 - fittingSize.height / 2
    )];

    _closeObserver = [[NSNotificationCenter defaultCenter]
        addObserverForName:NSWindowWillCloseNotification
                    object:_window
                     queue:nil
                usingBlock:^(NSNotification *__unused note) {
        stopRunLoop();
    }];

    void (*on_close)(void) = _on_close_requested;
    _monitor = [NSEvent addLocalMonitorForEventsMatchingMask:NSEventMaskKeyDown handler:^NSEvent *(NSEvent *event) {
        if (event.keyCode == 53 && on_close) { // escape
            on_close();
        }
        return event;
    }];

    [_window orderFrontRegardless];
}

- (void)closeWindow {
    [_window close];
}

- (void)dealloc {
    if (_monitor) {
        [NSEvent removeMonitor:_monitor];
    }
    if (_closeObserver) {
        [[NSNotificationCenter defaultCenter] removeObserver:_closeObserver];
    }
}

@end

void showSSOCode(const char *codeStr, void (*on_close_requested)(void)) {
    NSString *code = [NSString stringWithUTF8String:codeStr];
    NSApplication *app = [NSApplication sharedApplication];
    [app setActivationPolicy:NSApplicationActivationPolicyAccessory];
    appRunLoop = CFRunLoopGetCurrent();
    currentWindow = [[SSOCodeWindow alloc] init];
    [currentWindow showWithCode:code onCloseRequested:on_close_requested];
    [app run];
    currentWindow = nil;
    appRunLoop = nil;
}

void closeSSOCode(void) {
    CFRunLoopRef rl = appRunLoop;
    SSOCodeWindow *win = currentWindow;
    if (!rl || !win) return;
    CFRunLoopPerformBlock(rl, kCFRunLoopCommonModes, ^{
        [win closeWindow];
    });
    CFRunLoopWakeUp(rl);
}
