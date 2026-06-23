interface Props {
  /** Hours remaining in the veto window */
  hoursRemaining: number;
  proposalId: number | string;
}

/** Persistent amber banner shown to Guardian Multisig members when a proposal is pending. */
export function GovernanceAlert({ hoursRemaining, proposalId }: Props) {
  return (
    <div
      role="alert"
      style={{
        background: '#fefce8',
        border: '1px solid #fbbf24',
        borderRadius: 8,
        padding: '0.75rem 1rem',
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
        fontSize: '0.9rem',
        color: '#92400e',
      }}
    >
      <span aria-hidden="true">⚠️</span>
      <span>
        New governance proposal #{proposalId} open for veto — expires in{' '}
        <strong>{Math.round(hoursRemaining)} hours</strong>.
      </span>
    </div>
  );
}
