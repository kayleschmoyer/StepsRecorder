export type StepId = string;

export type Step = {
  id: StepId;
  sessionId: string;
  order: number;
  description: string;
};
