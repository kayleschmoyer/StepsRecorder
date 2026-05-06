import { invoke, type InvokeArgs } from '@tauri-apps/api/core';

type TauriCommandDefinition = {
  request: InvokeArgs | undefined;
  response: unknown;
};

type StepsRecorderCommands = {
  get_app_version: {
    request: undefined;
    response: string;
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
};
