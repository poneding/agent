import type { TraceTimeline, TraceUiState } from '../types';

export function createTraceState(conversationId: string | null): TraceUiState {
  return {
    conversationId,
    timeline:
      conversationId === null
        ? null
        : {
            conversation_id: conversationId,
            items: [],
          },
    loading: false,
    error: null,
  };
}

export function reduceTraceLoaded(
  state: TraceUiState,
  timeline: TraceTimeline,
): TraceUiState {
  return {
    ...state,
    conversationId: timeline.conversation_id,
    timeline,
    loading: false,
    error: null,
  };
}

export function reduceTraceError(state: TraceUiState, error: string): TraceUiState {
  return {
    ...state,
    loading: false,
    error,
  };
}
