package com.akashsamanta.manhua;

import android.accessibilityservice.AccessibilityService;
import android.util.Log;
import android.view.accessibility.AccessibilityEvent;
import android.view.accessibility.AccessibilityNodeInfo;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class BrowserWatcherService extends AccessibilityService {

    private static final String TAG = "BrowserWatcher";

    // crude but reliable: matches any http(s) URL text/content-description
    // anywhere in the node tree, so we don't depend on Brave's exact
    // address-bar resource id (which can change between Brave versions).
    // TODO: once confirmed via Layout Inspector, narrow this to the
    // specific node id for less tree-walking per event.
    private static final Pattern URL_PATTERN =
            Pattern.compile("https?://\\S+");

    // in-memory dedup — resets on service restart, which is fine, we'd
    // rather resend once on restart than never send at all.
    private String lastSentUrl = null;

    @Override
    public void onAccessibilityEvent(AccessibilityEvent event) {
        int type = event.getEventType();
        if (type != AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED
                && type != AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED) {
            return;
        }

        AccessibilityNodeInfo root = getRootInActiveWindow();
        if (root == null) return;

        String url = findUrlInTree(root);
        root.recycle();

        if (url == null) return;
        if (url.equals(lastSentUrl)) return; // same URL as last time, skip

        lastSentUrl = url;
        Log.d(TAG, "detected url: " + url);

        // TODO: domain allowlist check
        // TODO: queue locally (SQLite) instead of sending directly —
        //       flush happens on WorkManager tick, not from here
    }

    private String findUrlInTree(AccessibilityNodeInfo node) {
        if (node == null) return null;

        CharSequence text = node.getText();
        String fromText = extractUrl(text);
        if (fromText != null) return fromText;

        CharSequence desc = node.getContentDescription();
        String fromDesc = extractUrl(desc);
        if (fromDesc != null) return fromDesc;

        for (int i = 0; i < node.getChildCount(); i++) {
            AccessibilityNodeInfo child = node.getChild(i);
            if (child == null) continue;
            String found = findUrlInTree(child);
            child.recycle();
            if (found != null) return found;
        }

        return null;
    }

    private String extractUrl(CharSequence cs) {
        if (cs == null) return null;
        Matcher m = URL_PATTERN.matcher(cs);
        return m.find() ? m.group() : null;
    }

    @Override
    public void onInterrupt() {
        Log.w(TAG, "accessibility service interrupted");
    }

    @Override
    protected void onServiceConnected() {
        super.onServiceConnected();
        Log.i(TAG, "BrowserWatcherService connected");
    }
}
