import { Menu, Plugin, TAbstractFile, TFile, TFolder, MarkdownView } from 'obsidian';
import { addInsertCommentCommands } from './commands/commands';
import { Settings } from './settings/settings-type';
import {
    SIDEBAR_OUTLINE_VIEW_TYPE,
    SidebarOutlineView,
} from './sidebar-outline/sidebar-outline-view';
import { SettingsTab } from './settings/settings-tab/settings-tab';
import { Store } from './helpers/store';
import { SettingsActions, settingsReducer } from './settings/settings-reducer';
import { AnnotationSuggest } from './editor-suggest/annotation-suggest';
import { DEFAULT_SETTINGS } from './settings/default-settings';
import { tts } from './sidebar-outline/components/components/controls-bar/helpers/tts';
import { mergeDeep } from './settings/helpers/merge-objects';
import { registerEditorMenuEvent } from './note-creation/register-editor-menu-event';

import { OutlineUpdater } from './sidebar-outline/helpers/outline-updater/outline-updater';
import { loadOutlineStateFromSettings } from './settings/helpers/load-outline-state-from-settings';
import { subscribeSettingsToOutlineState } from './settings/helpers/subscribe-settings-to-outline-state';
import { StatusBar } from './status-bar/status-bar';
import { fileMenuItems } from './clipboard/file-menu-items';
import { subscribeDecorationStateToSettings } from './settings/helpers/subscribe-decoration-state-to-settings';
import { DecorationSettings } from './editor-plugin/helpers/decorate-annotations/decoration-settings';
import { EditorPlugin, editorPlugin } from './editor-plugin/editor-plugin';
import { Idling } from './idling/idling';

// Stubs imports
import {
    registerStubsCommands,
    stubsEditorPlugin,
    StubsEditorPlugin,
    updateStubsConfig,
    updateSyncState,
    performSync,
    stubAnchorStyles,
    StubSuggest,
    stubSuggestStyles,
} from './stubs';

export default class LabeledAnnotations extends Plugin {
    outline: OutlineUpdater;
    settings: Store<Settings, SettingsActions>;
    statusBar: StatusBar;
    idling: Idling;
    decorationSettings: DecorationSettings;
    editorSuggest: AnnotationSuggest;
    stubSuggest: StubSuggest;
    private unsubscribeCallbacks: Set<() => void> = new Set();

    async onload() {
        await this.loadSettings();

        this.editorSuggest = new AnnotationSuggest(this.app, this);
        this.registerEditorSuggest(this.editorSuggest);
        this.registerEvent(
            this.app.workspace.on(
                'file-menu',
                (menu: Menu, abstractFiles: TAbstractFile) => {
                    if (
                        abstractFiles instanceof TFolder ||
                        (abstractFiles instanceof TFile &&
                            abstractFiles.extension === 'md')
                    )
                        fileMenuItems(this)(menu, abstractFiles);
                },
            ),
        );
        this.registerEvent(
            this.app.workspace.on('files-menu', fileMenuItems(this)),
        );

        addInsertCommentCommands(this);

        // Register stubs commands
        try {
            registerStubsCommands(this);
        } catch (error) {
            console.error('Failed to register stubs commands:', error);
        }

        this.registerView(
            SIDEBAR_OUTLINE_VIEW_TYPE,
            (leaf) => new SidebarOutlineView(leaf, this),
        );

        this.app.workspace.onLayoutReady(async () => {
            await this.attachLeaf();
            loadOutlineStateFromSettings(this);
            this.registerSubscription(...subscribeSettingsToOutlineState(this));
            this.addSettingTab(new SettingsTab(this.app, this));
            registerEditorMenuEvent(this);
            this.outline = new OutlineUpdater(this);
            this.statusBar = new StatusBar(this);
            tts.setPlugin(this);
            this.idling = new Idling(this);
        });
    }

    onunload() {
        tts.stop();
        for (const callback of this.unsubscribeCallbacks) {
            callback();
        }
    }

    loadPlugin() {
        this.decorationSettings = new DecorationSettings(this);
        EditorPlugin.plugin = this;
        this.unsubscribeCallbacks.add(subscribeDecorationStateToSettings(this));
        this.registerEditorExtension([editorPlugin]);

        // Initialize stubs
        this.initializeStubs();
    }

    /**
     * Initialize stubs module
     */
    initializeStubs() {
        try {
            // Set stubs editor plugin reference
            StubsEditorPlugin.plugin = this;

            // Register stubs editor extension
            this.registerEditorExtension([stubsEditorPlugin]);

            // Register stub suggest (^^ trigger)
            this.stubSuggest = new StubSuggest(this.app, this);
            this.registerEditorSuggest(this.stubSuggest);

            // Initialize stubs config from settings
            updateStubsConfig(this.settings.getValue().stubs);

            // Subscribe to settings changes to update stubs config
            this.unsubscribeCallbacks.add(
                this.settings.subscribe(() => {
                    updateStubsConfig(this.settings.getValue().stubs);
                })
            );

            // Register event to sync stubs when file changes
            this.registerEvent(
                this.app.workspace.on('active-leaf-change', () => {
                    this.syncStubsForActiveFile();
                })
            );

            this.registerEvent(
                this.app.metadataCache.on('changed', (file) => {
                    const activeFile = this.app.workspace.getActiveFile();
                    if (activeFile && file.path === activeFile.path) {
                        this.syncStubsForActiveFile();
                    }
                })
            );

            // Add CSS for stub decorations and suggest styles
            this.addStubStyles();
        } catch (error) {
            console.error('Failed to initialize stubs module:', error);
        }
    }

    /**
     * Sync stubs for the active file
     */
    async syncStubsForActiveFile() {
        const view = this.app.workspace.getActiveViewOfType(MarkdownView);
        if (!view || !view.file) {
            return;
        }

        const config = this.settings.getValue().stubs;
        if (!config.enabled) {
            return;
        }

        try {
            const content = await this.app.vault.read(view.file);
            const result = await performSync(this.app, view.file, content, config);
            updateSyncState(result);
        } catch (error) {
            console.error('Failed to sync stubs:', error);
        }
    }

    /**
     * Add CSS styles for stub anchor decorations and suggest dropdown
     */
    addStubStyles() {
        const styleEl = document.createElement('style');
        styleEl.id = 'm-stubs-styles';
        styleEl.textContent = stubAnchorStyles + '\n' + stubSuggestStyles;
        document.head.appendChild(styleEl);

        // Clean up on unload
        this.register(() => {
            const el = document.getElementById('m-stubs-styles');
            if (el) {
                el.remove();
            }
        });
    }

    async loadSettings() {
        const settings = (await this.loadData()) || {};
        this.settings = new Store<Settings, SettingsActions>(
            mergeDeep(settings, DEFAULT_SETTINGS()),
            settingsReducer,
        );
        this.registerSubscription(
            this.settings.subscribe(() => {
                this.saveSettings();
            }),
        );
    }

    async saveSettings() {
        await this.saveData(this.settings.getValue());
    }

    async attachLeaf() {
        const leaves = this.app.workspace.getLeavesOfType(
            SIDEBAR_OUTLINE_VIEW_TYPE,
        );
        if (leaves.length === 0) {
            await this.app.workspace.getRightLeaf(false).setViewState({
                type: SIDEBAR_OUTLINE_VIEW_TYPE,
                active: true,
            });
        }
    }

    async revealLeaf() {
        const leaf = this.app.workspace.getLeavesOfType(
            SIDEBAR_OUTLINE_VIEW_TYPE,
        )[0];
        if (leaf) this.app.workspace.revealLeaf(leaf);
        else {
            await this.attachLeaf();
            await this.revealLeaf();
        }
    }

    registerSubscription(...callback: (() => void)[]) {
        callback.forEach((callback) => {
            this.unsubscribeCallbacks.add(callback);
        });
    }
}
