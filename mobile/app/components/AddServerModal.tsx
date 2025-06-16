import React from 'react';
import {Server} from '../services/serverState';
import {ServerFormModal} from './ServerFormModal';

interface AddServerModalProps {
  visible: boolean;
  onClose: () => void;
  addServer: (
    server: Omit<Server, 'status' | 'data' | 'autoStart'>,
  ) => Promise<void>;
}

export const AddServerModal: React.FC<AddServerModalProps> = ({
  visible,
  onClose,
  addServer,
}) => {
  return (
    <ServerFormModal
      visible={visible}
      onClose={onClose}
      onSubmit={addServer}
      title="Add Server"
      submitButtonText="Save"
    />
  );
};
