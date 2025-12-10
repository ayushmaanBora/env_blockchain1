# This is a template for creating the "Yuki Vision" model
import tensorflow as tf
from tensorflow.keras import layers, models

def create_model():
    model = models.Sequential([
        # Resize images to standard input
        layers.Rescaling(1./255, input_shape=(180, 180, 3)),
        
        # Convolutional Layers to find "features" (leaves, plastic, etc)
        layers.Conv2D(32, (3, 3), activation='relu'),
        layers.MaxPooling2D((2, 2)),
        layers.Conv2D(64, (3, 3), activation='relu'),
        layers.MaxPooling2D((2, 2)),
        
        layers.Flatten(),
        layers.Dense(64, activation='relu'),
        # Output: 3 Classes (Tree, Plastic, Other)
        layers.Dense(3) 
    ])
    
    model.compile(optimizer='adam',
                  loss=tf.keras.losses.SparseCategoricalCrossentropy(from_logits=True),
                  metrics=['accuracy'])
    return model

print("To train this, you would need a dataset folder like:")
print("/dataset/trees/")
print("/dataset/plastic/")
print("Then use: model.fit(train_ds, validation_data=val_ds, epochs=10)")