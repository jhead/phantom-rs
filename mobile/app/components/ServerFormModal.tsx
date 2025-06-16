import React, {useState, useEffect, useRef} from 'react';
import {
  Modal,
  View,
  Text,
  TextInput,
  TouchableOpacity,
  Animated,
  Dimensions,
  Easing,
  Keyboard,
  TouchableWithoutFeedback,
} from 'react-native';
import {styles} from './ServersScreen.styles';
import {Server} from '../services/serverState';

const defaultInitialValues = {
  name: 'MC Complex',
  address: 'mps.mc-complex.com',
  port: '19132',
};

interface ServerFormModalProps {
  visible: boolean;
  onClose: () => void;
  onSubmit: (
    server: Omit<Server, 'status' | 'data' | 'autoStart'>,
  ) => Promise<void>;
  initialValues?: {
    name: string;
    address: string;
    port: string;
  };
  title: string;
  submitButtonText: string;
}

export const ServerFormModal: React.FC<ServerFormModalProps> = ({
  visible,
  onClose,
  onSubmit,
  initialValues = defaultInitialValues,
  title,
  submitButtonText,
}) => {
  const [name, setName] = useState(initialValues.name);
  const [address, setAddress] = useState(initialValues.address);
  const [port, setPort] = useState(initialValues.port);
  const [errors, setErrors] = useState<{
    name?: string;
    address?: string;
    port?: string;
  }>({});
  const translateY = useRef(
    new Animated.Value(Dimensions.get('window').height),
  ).current;
  const opacityAnim = useRef(new Animated.Value(0)).current;
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Reset form state when the modal becomes visible or initial values change
  useEffect(() => {
    if (visible) {
      setName(initialValues.name);
      setAddress(initialValues.address);
      setPort(initialValues.port);
      setErrors({});
    }
  }, [visible, initialValues]);

  // Cleanup animation values on unmount
  useEffect(() => {
    return () => {
      translateY.setValue(Dimensions.get('window').height);
      opacityAnim.setValue(0);
    };
  }, []);

  const validateForm = () => {
    const newErrors: typeof errors = {};

    if (!name.trim()) {
      newErrors.name = 'Name is required';
    }

    if (!address.trim()) {
      newErrors.address = 'Address is required';
    }

    const portNum = parseInt(port, 10);
    if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
      newErrors.port = 'Port must be between 1 and 65535';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async () => {
    if (!validateForm() || isSubmitting) {
      return;
    }

    setIsSubmitting(true);
    const serverData: Omit<Server, 'status' | 'data' | 'autoStart'> = {
      id: Date.now().toString(), // This will be overridden by the parent component for edit mode
      name: name.trim(),
      address: address.trim(),
      port: port.trim(),
    };

    try {
      console.log('Submitting server:', serverData);
      await onSubmit(serverData);
      console.log('Server submitted successfully:', serverData);

      // Reset form and close modal
      setName(initialValues.name);
      setAddress(initialValues.address);
      setPort(initialValues.port);
      setErrors({});
      onClose();
    } catch (error) {
      console.error('Failed to submit server:', error);
      setErrors(prev => ({
        ...prev,
        address:
          'Failed to connect to server. Please check the address and try again.',
      }));
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDismiss = () => {
    if (isSubmitting) return;
    Keyboard.dismiss();
    onClose();
  };

  useEffect(() => {
    if (visible) {
      Animated.parallel([
        Animated.timing(translateY, {
          toValue: 0,
          duration: 300,
          useNativeDriver: true,
          easing: Easing.bezier(0.25, 0.1, 0.25, 1),
        }),
        Animated.timing(opacityAnim, {
          toValue: 1,
          duration: 300,
          useNativeDriver: true,
        }),
      ]).start();
    } else {
      Animated.parallel([
        Animated.timing(translateY, {
          toValue: Dimensions.get('window').height,
          duration: 250,
          useNativeDriver: true,
          easing: Easing.bezier(0.25, 0.1, 0.25, 1),
        }),
        Animated.timing(opacityAnim, {
          toValue: 0,
          duration: 250,
          useNativeDriver: true,
        }),
      ]).start();
    }
  }, [visible, translateY, opacityAnim]);

  return (
    <Modal
      visible={visible}
      transparent
      animationType="none"
      onRequestClose={handleDismiss}>
      <TouchableWithoutFeedback onPress={handleDismiss}>
        <Animated.View
          style={[
            styles.modalOverlay,
            {
              opacity: opacityAnim,
            },
          ]}>
          <TouchableWithoutFeedback>
            <Animated.View
              style={[
                styles.modalContainer,
                {
                  transform: [{translateY}],
                },
              ]}>
              <Text style={styles.modalTitle}>{title}</Text>
              <Text style={styles.modalLabel}>Name</Text>
              <TextInput
                style={[styles.input, errors.name && styles.inputError]}
                placeholder="Example Server"
                placeholderTextColor="#aaa"
                value={name}
                onChangeText={text => {
                  setName(text);
                  if (errors.name) {
                    setErrors(prev => ({...prev, name: undefined}));
                  }
                }}
                returnKeyType="next"
                editable={!isSubmitting}
              />
              {errors.name && (
                <Text style={styles.errorText}>{errors.name}</Text>
              )}
              <Text style={styles.modalLabel}>Address</Text>
              <TextInput
                style={[styles.input, errors.address && styles.inputError]}
                placeholder="mc.example.com"
                placeholderTextColor="#aaa"
                value={address}
                onChangeText={text => {
                  setAddress(text);
                  if (errors.address) {
                    setErrors(prev => ({...prev, address: undefined}));
                  }
                }}
                returnKeyType="next"
                autoCapitalize="none"
                editable={!isSubmitting}
              />
              {errors.address && (
                <Text style={styles.errorText}>{errors.address}</Text>
              )}
              <Text style={styles.modalLabel}>Port</Text>
              <TextInput
                style={[styles.input, errors.port && styles.inputError]}
                placeholder="19132"
                placeholderTextColor="#aaa"
                value={port}
                onChangeText={text => {
                  setPort(text);
                  if (errors.port) {
                    setErrors(prev => ({...prev, port: undefined}));
                  }
                }}
                keyboardType="numeric"
                returnKeyType="done"
                onSubmitEditing={handleSubmit}
                editable={!isSubmitting}
              />
              {errors.port && (
                <Text style={styles.errorText}>{errors.port}</Text>
              )}
              <View style={styles.modalButtonRow}>
                <TouchableOpacity
                  style={[
                    styles.saveButton,
                    isSubmitting && styles.saveButtonDisabled,
                  ]}
                  onPress={handleSubmit}
                  disabled={isSubmitting}>
                  <Text style={styles.saveButtonText}>
                    {isSubmitting ? 'Saving...' : submitButtonText}
                  </Text>
                </TouchableOpacity>
                <TouchableOpacity
                  style={[
                    styles.cancelButton,
                    isSubmitting && styles.cancelButtonDisabled,
                  ]}
                  onPress={handleDismiss}
                  disabled={isSubmitting}>
                  <Text style={styles.cancelButtonText}>Cancel</Text>
                </TouchableOpacity>
              </View>
            </Animated.View>
          </TouchableWithoutFeedback>
        </Animated.View>
      </TouchableWithoutFeedback>
    </Modal>
  );
};
