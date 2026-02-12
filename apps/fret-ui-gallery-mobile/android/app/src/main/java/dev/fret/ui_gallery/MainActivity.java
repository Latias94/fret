package dev.fret.ui_gallery;

import com.google.androidgamesdk.GameActivity;

public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("fret_ui_gallery_mobile");
    }
}

