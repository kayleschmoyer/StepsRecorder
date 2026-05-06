import { invoke, type InvokeArgs } from '@tauri-apps/api/core';

export interface AppErrorResponse {
  code: string;
  message: string;
  details?: string;
}

export interface ListSessionsInput {
  limit?: number;
  includeArchived?: boolean;
}

export interface SessionSummary {
  id: string;
  title: string;
  status: string;
  startedAt: string;
  endedAt?: string;
  stepCount: number;
}

export interface GetSessionInput {
  sessionId: string;
}

export interface StartRecordingSessionInput {
  title?: string;
  description?: string;
}

export interface StopRecordingSessionInput {
  sessionId: string;
}

export interface RecordingStatus {
  isRecording: boolean;
  activeSessionId?: string;
  elapsedSeconds?: number;
  stepCount: number;
}

export interface ClearSeededDataResult {
  sessionId: string;
  deletedSessions: number;
  deletedSteps: number;
}

export interface ListScreenshotEditsInput {
  stepId: string;
}

export interface GetStepScreenshotPreviewInput {
  stepId: string;
}

export interface StepScreenshotPreview {
  exists: boolean;
  originalScreenshotPath: string;
  editedScreenshotPath?: string;
  displayedScreenshotPath?: string;
  previewKind: 'missing' | 'original' | 'click_marker' | 'edited';
  dataUrl?: string;
}

export interface SaveEditedScreenshotInput {
  stepId: string;
  screenshotDataUrl: string;
}

export interface ScreenshotEdit {
  id: string;
  stepId: string;
  editType: 'crop' | 'redact' | 'highlight' | 'arrow' | 'text';
  editDataJson: string;
  createdAt: string;
}

export interface RecordingStep {
  id: string;
  sessionId: string;
  stepNumber: number;
  title: string;
  description?: string;
  actionType: 'click';
  capturedAt: string;
  clickX?: number;
  clickY?: number;
  monitorId?: string;
  appWindowTitle?: string;
  processName?: string;
  originalScreenshotPath: string;
  editedScreenshotPath?: string;
  thumbnailPath?: string;
  isDeleted: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface SessionDetail {
  id: string;
  title: string;
  description?: string;
  status: string;
  startedAt: string;
  endedAt?: string;
  steps: RecordingStep[];
}

export interface UpdateSessionInput {
  sessionId: string;
  title?: string;
  description?: string;
  includeTimestampsDefault?: boolean;
  includeClickMarkersDefault?: boolean;
}

export interface UpdateStepInput {
  stepId: string;
  title?: string;
  description?: string;
}

export interface DeleteStepInput {
  stepId: string;
}

export interface DeleteStepResult {
  stepId: string;
  sessionId: string;
  deleted: boolean;
}

export interface ReorderStepsInput {
  sessionId: string;
  orderedStepIds: string[];
}

export interface ReorderStepsResult {
  sessionId: string;
  steps: RecordingStep[];
}

export interface RecordingSession {
  id: string;
  title: string;
  description?: string;
  status: string;
  startedAt: string;
  endedAt?: string;
  createdAt: string;
  updatedAt: string;
  defaultExportDirectory?: string;
  stepCount: number;
  includeTimestampsDefault: boolean;
  includeClickMarkersDefault: boolean;
}

export type ScreenshotMode = 'clicked_monitor' | 'clicked_window';

export interface AppSettings {
  screenshotMode: ScreenshotMode;
  clickDebounceMs: number;
  includeTimestampsInExport: boolean;
  includeClickMarkers: boolean;
  privacyReminderBeforeExport: boolean;
  defaultExportDirectory?: string;
}

export interface UpdateSettingsInput {
  screenshotMode?: ScreenshotMode;
  clickDebounceMs?: number;
  includeTimestampsInExport?: boolean;
  includeClickMarkers?: boolean;
  privacyReminderBeforeExport?: boolean;
  defaultExportDirectory?: string;
}

export interface ListExportHistoryInput {
  sessionId: string;
}

export interface ExportHistoryRecord {
  id: string;
  sessionId: string;
  exportType: 'docx' | 'pdf';
  outputPath: string;
  exportedAt: string;
  includeTimestamps: boolean;
  includeClickMarkers: boolean;
  status: 'success' | 'failed';
  errorMessage?: string;
}

type TauriCommandDefinition = {
  request: InvokeArgs | undefined;
  response: unknown;
};

type StepsRecorderCommands = {
  get_app_version: {
    request: undefined;
    response: string;
  };
  get_settings: {
    request: undefined;
    response: AppSettings;
  };
  update_settings: {
    request: { input: UpdateSettingsInput };
    response: AppSettings;
  };
  start_recording_session: {
    request: { input: StartRecordingSessionInput };
    response: RecordingSession;
  };
  stop_recording_session: {
    request: { input: StopRecordingSessionInput };
    response: RecordingSession;
  };
  get_recording_status: {
    request: undefined;
    response: RecordingStatus;
  };
  list_sessions: {
    request: { input?: ListSessionsInput } | undefined;
    response: SessionSummary[];
  };
  get_session: {
    request: { input: GetSessionInput };
    response: SessionDetail;
  };
  get_step_screenshot_preview: {
    request: { input: GetStepScreenshotPreviewInput };
    response: StepScreenshotPreview;
  };
  save_edited_screenshot: {
    request: { input: SaveEditedScreenshotInput };
    response: RecordingStep;
  };
  update_session: {
    request: { input: UpdateSessionInput };
    response: RecordingSession;
  };
  update_step: {
    request: { input: UpdateStepInput };
    response: RecordingStep;
  };
  delete_step: {
    request: { input: DeleteStepInput };
    response: DeleteStepResult;
  };
  reorder_steps: {
    request: { input: ReorderStepsInput };
    response: ReorderStepsResult;
  };
  list_screenshot_edits: {
    request: { input: ListScreenshotEditsInput };
    response: ScreenshotEdit[];
  };
  list_export_history: {
    request: { input: ListExportHistoryInput };
    response: ExportHistoryRecord[];
  };
  dev_seed_sample_data: {
    request: undefined;
    response: SessionDetail;
  };
  dev_clear_seeded_data: {
    request: undefined;
    response: ClearSeededDataResult;
  };
};

type CommandMap = Record<string, TauriCommandDefinition>;

type AppCommandMap = StepsRecorderCommands & CommandMap;

export type TauriCommandName = keyof StepsRecorderCommands;

export async function invokeTauriCommand<Name extends TauriCommandName>(
  commandName: Name,
  request: AppCommandMap[Name]['request'],
): Promise<AppCommandMap[Name]['response']> {
  return invoke<AppCommandMap[Name]['response']>(commandName, request);
}

export const tauriClient = {
  getAppVersion: () => invokeTauriCommand('get_app_version', undefined),
  getSettings: () => invokeTauriCommand('get_settings', undefined),
  updateSettings: (input: UpdateSettingsInput) => invokeTauriCommand('update_settings', { input }),
  startRecordingSession: (input: StartRecordingSessionInput = {}) => invokeTauriCommand('start_recording_session', { input }),
  stopRecordingSession: (input: StopRecordingSessionInput) => invokeTauriCommand('stop_recording_session', { input }),
  getRecordingStatus: () => invokeTauriCommand('get_recording_status', undefined),
  listSessions: (input?: ListSessionsInput) => invokeTauriCommand('list_sessions', input ? { input } : undefined),
  getSession: (input: GetSessionInput) => invokeTauriCommand('get_session', { input }),
  getStepScreenshotPreview: (input: GetStepScreenshotPreviewInput) => invokeTauriCommand('get_step_screenshot_preview', { input }),
  saveEditedScreenshot: (input: SaveEditedScreenshotInput) => invokeTauriCommand('save_edited_screenshot', { input }),
  updateSession: (input: UpdateSessionInput) => invokeTauriCommand('update_session', { input }),
  updateStep: (input: UpdateStepInput) => invokeTauriCommand('update_step', { input }),
  deleteStep: (input: DeleteStepInput) => invokeTauriCommand('delete_step', { input }),
  reorderSteps: (input: ReorderStepsInput) => invokeTauriCommand('reorder_steps', { input }),
  listScreenshotEdits: (input: ListScreenshotEditsInput) => invokeTauriCommand('list_screenshot_edits', { input }),
  listExportHistory: (input: ListExportHistoryInput) => invokeTauriCommand('list_export_history', { input }),
  devSeedSampleData: () => invokeTauriCommand('dev_seed_sample_data', undefined),
  devClearSeededData: () => invokeTauriCommand('dev_clear_seeded_data', undefined),
};
