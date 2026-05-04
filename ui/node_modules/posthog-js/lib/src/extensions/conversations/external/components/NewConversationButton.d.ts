import { FunctionComponent } from 'preact';
import { getStyles } from './styles';
interface NewConversationButtonProps {
    styles: ReturnType<typeof getStyles>;
    onClick: () => void;
}
/**
 * Primary CTA used anywhere the user can start a fresh conversation —
 * the bottom of the ticket list and the resolved-state banner in the message view.
 */
export declare const NewConversationButton: FunctionComponent<NewConversationButtonProps>;
export {};
