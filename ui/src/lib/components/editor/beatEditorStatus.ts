export function beatContentStatusLabel(status: string): string {
  switch (status) {
    case 'Empty':
      return 'No content';
    case 'NotesOnly':
      return 'Notes written';
    case 'Generating':
      return 'AI generating...';
    case 'HasContent':
      return 'Has content';
    default:
      return status;
  }
}
