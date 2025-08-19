import Checkbox, { CheckboxProps } from "@mui/material/Checkbox";
import FormControlLabel from "@mui/material/FormControlLabel";
import { styled } from "@mui/material/styles";
import { FieldProps, FieldRenderProps } from "react-final-form";

import { FieldWrapper } from "./fieldWrapper";

import { CssGrid, Orientation, Position } from "../cssGrid";

const ClearFieldWrapper = styled(CssGrid)`
    position: relative;
`;

const FieldCheckbox = ({
    CheckboxProps = {},
    label,
    ...fieldProps
}: FieldCheckboxProps): JSX.Element => {
    return (
        <ClearFieldWrapper
            orientation={Orientation.horizontal}
            template="auto min-content"
            alignItems={Position.start}
        >
            <FieldWrapper {...fieldProps} type="checkbox">
                {({ input }: FieldWrapperRenderProps) => {
                    const { onFocus, onBlur, ...remainingInputProps } = input;
                    const checkbox = (
                        <Checkbox
                            id={fieldProps.name}
                            inputProps={{ onFocus, onBlur }}
                            {...remainingInputProps}
                            color="primary"
                            {...CheckboxProps}
                            sx={{
                                ...(CheckboxProps?.sx || {}),
                            }}
                        />
                    );
                    return (
                        <FormControlLabel control={checkbox} label={label} />
                    );
                }}
            </FieldWrapper>
        </ClearFieldWrapper>
    );
};

type FieldWrapperRenderProps = FieldRenderProps<boolean, HTMLInputElement>;
interface FieldCheckboxProps
    extends FieldProps<boolean, FieldWrapperRenderProps, HTMLInputElement> {
    CheckboxProps?: Partial<CheckboxProps>;
}

export default FieldCheckbox;
