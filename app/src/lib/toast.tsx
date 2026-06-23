import { toast as sonnerToast } from 'sonner';

const EXPLORER = 'https://stellar.expert/explorer/public/tx';

export const toast = {
  submitting: (msg = 'Submitting transaction…') =>
    sonnerToast.loading(msg),

  confirmed: (txHash?: string) =>
    sonnerToast.success('Transaction confirmed', {
      description: txHash ? (
        <a href={`${EXPLORER}/${txHash}`} target="_blank" rel="noopener noreferrer">
          View on Stellar Expert ↗
        </a>
      ) : undefined,
    }),

  failed: (reason?: string) =>
    sonnerToast.error(`Transaction failed${reason ? `: ${reason}` : ''}`)  ,

  harvest: () =>
    sonnerToast.success('Yield harvested — your position has been compounded.'),

  dismiss: sonnerToast.dismiss,
};
